//! Propriedades da conversão aplicação→timeline. A moldura e a profundidade
//! são verificadas antes desta camada pelos leitores de `service/mod.rs`.

use super::timeline_event_from_item;
use chrono::{DateTime, SecondsFormat, Utc};
use dfir_protocol::{ITEM_KIND_DETECTION_CANDIDATE, NormalizedEvent, normalize};
use proptest::{collection, prelude::*, test_runner::TestCaseResult};
use serde_json::{Value, json};

fn json_values() -> BoxedStrategy<Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|number| Value::Number(number.into())),
        any::<String>().prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 24, 3, |inner| {
        prop_oneof![
            collection::vec(inner.clone(), 0..3).prop_map(Value::Array),
            collection::btree_map(any::<String>(), inner, 0..3)
                .prop_map(|entries| { Value::Object(entries.into_iter().collect()) }),
        ]
    })
    .boxed()
}

fn texts() -> BoxedStrategy<String> {
    collection::vec(any::<char>(), 0..20)
        .prop_map(|characters| characters.into_iter().collect())
        .boxed()
}

fn timestamps() -> BoxedStrategy<String> {
    (0..=4_102_444_800_i64, 0..1_000_000_000_u32, 0..3_u8)
        .prop_map(|(seconds, nanoseconds, format)| {
            let timestamp = DateTime::from_timestamp(seconds, nanoseconds).expect("in-range time");
            match format {
                0 => timestamp.to_rfc3339(),
                1 => timestamp.to_rfc3339_opts(SecondsFormat::Nanos, true),
                _ => timestamp.to_rfc3339_opts(SecondsFormat::Millis, false),
            }
        })
        .boxed()
}

fn event_data() -> BoxedStrategy<Value> {
    (
        any::<u32>(),
        0..3_u8,
        prop::option::of(texts()),
        any::<bool>(),
        prop::option::of(texts()),
        prop::option::of(texts()),
        json_values(),
    )
        .prop_map(
            |(id, id_form, provider, attributes, channel, computer, future)| {
                let event_id = match id_form {
                    0 => json!(id),
                    1 => json!(id.to_string()),
                    _ => json!({"#attributes": {"Qualifiers": 16384}, "#text": id}),
                };
                let mut system = json!({"EventID": event_id, "FutureSystemField": future});
                if let Some(provider) = provider {
                    system["Provider"] = if attributes {
                        json!({"#attributes": {"Name": provider}})
                    } else {
                        json!({"Name": provider})
                    };
                }
                if let Some(channel) = channel {
                    system["Channel"] = json!(channel);
                }
                if let Some(computer) = computer {
                    system["Computer"] = json!(computer);
                }
                json!({"Event": {"System": system}})
            },
        )
        .boxed()
}

fn normalized_events() -> BoxedStrategy<(Value, Value)> {
    (
        any::<u64>(),
        timestamps(),
        event_data(),
        "[A-Za-z0-9_-]{1,24}",
        any::<bool>(),
        json_values(),
    )
        .prop_map(
            |(record_id, timestamp, data, evidence_id, structured, future)| {
                let event = if structured {
                    let mut fields = normalize::event_fields(&data);
                    fields.source_type = Some("EVTX".into());
                    NormalizedEvent::new(record_id, timestamp, data.clone(), evidence_id)
                        .with_fields(fields)
                } else {
                    NormalizedEvent::new(record_id, timestamp, data.clone(), evidence_id)
                };
                let mut item = event.into_item().expect("generated event serializes");
                item.as_object_mut()
                    .expect("event object")
                    .insert("future_extension".into(), future);
                (item, data)
            },
        )
        .boxed()
}

fn reordered_events() -> BoxedStrategy<(Value, Value)> {
    normalized_events()
        .prop_flat_map(|(item, _)| {
            let entries = item
                .as_object()
                .expect("event object")
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<Vec<_>>();
            let keys = collection::vec(any::<u64>(), entries.len());
            (Just((item, entries)), keys)
        })
        .prop_map(|((original, entries), keys)| {
            let mut keyed = entries.into_iter().zip(keys).collect::<Vec<_>>();
            keyed.sort_by_key(|((_, _), key)| *key);
            let body = keyed
                .into_iter()
                .map(|((key, value), _)| {
                    format!("{}:{value}", serde_json::to_string(&key).expect("JSON key"))
                })
                .collect::<Vec<_>>()
                .join(",");
            let reordered = serde_json::from_str::<Value>(&format!("{{{body}}}"))
                .expect("shuffled object is valid JSON");
            (original, reordered)
        })
        .boxed()
}

fn assert_presentation_invariants(item: &Value, data: &Value) -> TestCaseResult {
    let normalized = NormalizedEvent::from_item(item).expect("generated normalized event");
    let event = timeline_event_from_item(item).expect("generated timeline event");
    let expected_fields = normalize::event_fields(data);
    let expected_sort = DateTime::parse_from_rfc3339(&normalized.timestamp)
        .expect("generated RFC3339 timestamp")
        .with_timezone(&Utc);

    prop_assert_eq!(event.sort_timestamp, expected_sort);
    prop_assert_eq!(event.timestamp, normalized.timestamp);
    prop_assert_eq!(
        event.id,
        format!("{}:{}", normalized.evidence_id, normalized.raw_reference)
    );
    prop_assert_eq!(
        event.record_id,
        Some(normalized.event_record_id.to_string())
    );
    prop_assert_eq!(
        event.evidence_id.as_deref(),
        Some(normalized.evidence_id.as_str())
    );
    prop_assert_eq!(
        event.raw_reference.as_deref(),
        Some(normalized.raw_reference.as_str())
    );
    prop_assert_eq!(event.event_id, expected_fields.event_id);
    prop_assert_eq!(event.provider, expected_fields.provider);
    prop_assert_eq!(event.channel, expected_fields.channel);
    prop_assert_eq!(event.host, expected_fields.host);
    prop_assert_eq!(event.user, expected_fields.user);
    prop_assert_eq!(event.src_ip, expected_fields.src_ip);
    prop_assert_eq!(event.dst_ip, expected_fields.dst_ip);
    prop_assert_eq!(event.process, expected_fields.process);
    prop_assert_eq!(event.action, expected_fields.action);
    prop_assert_eq!(event.outcome, expected_fields.outcome);
    prop_assert_eq!(event.source_type, expected_fields.source_type);
    prop_assert_eq!(event.category, "EVTX");
    prop_assert!(event.description.contains("Event ID"));
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn arbitrary_bounded_items_never_panic(item in json_values()) {
        let _converted = timeline_event_from_item(&item);
    }

    #[test]
    fn current_legacy_and_unknown_fields_preserve_presentation(
        (item, data) in normalized_events()
    ) {
        assert_presentation_invariants(&item, &data)?;
    }

    #[test]
    fn key_order_does_not_change_presentation((original, reordered) in reordered_events()) {
        let expected = timeline_event_from_item(&original).expect("base event");
        let actual = timeline_event_from_item(&reordered).expect("reordered event");
        prop_assert_eq!(format!("{expected:?}"), format!("{actual:?}"));
    }

    #[test]
    fn explicit_non_timeline_kinds_are_rejected(
        (item, _) in normalized_events(),
        kind in prop_oneof![
            Just("YARA_MATCH"),
            Just(ITEM_KIND_DETECTION_CANDIDATE),
            Just("FINDING"),
        ],
    ) {
        let mut other = item.clone();
        other.as_object_mut()
            .expect("event object")
            .insert("kind".into(), json!(kind));
        prop_assert!(timeline_event_from_item(&other).is_none(), "accepted kind {kind}");

        let mut normalized = item;
        normalized
            .as_object_mut()
            .expect("event object")
            .insert("kind".into(), json!("NORMALIZED_EVENT"));
        prop_assert!(timeline_event_from_item(&normalized).is_some());
    }

    #[test]
    fn invalid_required_fields_and_timestamps_are_rejected(
        (item, _) in normalized_events(),
        mutation in 0_u8..10,
    ) {
        let mut invalid = item;
        if mutation == 9 {
            invalid = Value::Null;
        } else {
            let object = invalid.as_object_mut().expect("event object");
            match mutation {
                0 => { object.remove("event_record_id"); }
                1 => { object.insert("event_record_id".into(), json!("1")); }
                2 => { object.remove("timestamp"); }
                3 => { object.insert("timestamp".into(), json!("not-a-date")); }
                4 => { object.insert("evidence_id".into(), json!(1)); }
                5 => { object.insert("raw_reference".into(), Value::Null); }
                6 => { object.remove("data"); }
                7 => { object.insert("host".into(), json!(42)); }
                8 => { object.insert("timestamp".into(), json!(1)); }
                _ => unreachable!(),
            }
        }
        prop_assert!(timeline_event_from_item(&invalid).is_none());
    }
}
