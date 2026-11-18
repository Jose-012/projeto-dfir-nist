use dfir_protocol::{
    ANALYSIS_KIND_EVTX_TIMELINE, EventFields, MAX_LINE_BYTES, NormalizedEvent, PROTOCOL_VERSION,
    SCHEMA_VERSION, json_depth_exceeds_limit, validate,
};
use proptest::{collection, prelude::*};
use serde_json::{Value, json};

fn json_values() -> BoxedStrategy<Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|number| Value::Number(number.into())),
        any::<String>().prop_map(Value::String),
    ];
    leaf.prop_recursive(4, 32, 3, |inner| {
        prop_oneof![
            collection::vec(inner.clone(), 0..3).prop_map(Value::Array),
            collection::btree_map(any::<String>(), inner, 0..3)
                .prop_map(|entries| { Value::Object(entries.into_iter().collect()) }),
        ]
    })
    .boxed()
}

fn json_depth(value: &Value) -> usize {
    match value {
        Value::Array(items) => 1 + items.iter().map(json_depth).max().unwrap_or(0),
        Value::Object(fields) => 1 + fields.values().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn logical_references() -> BoxedStrategy<String> {
    collection::vec("[A-Za-z0-9_][A-Za-z0-9._-]{0,15}", 1..4)
        .prop_map(|parts| format!("evidence/{}", parts.join("/")))
        .boxed()
}

fn request_json(input_reference: &str) -> Value {
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "schema_version": SCHEMA_VERSION,
        "message_type": "REQUEST",
        "request_id": "req",
        "job_id": "job",
        "input_reference": input_reference,
        "output_reference": null,
        "max_output_bytes": 1024,
        "max_items": 10,
        "analysis_kind": ANALYSIS_KIND_EVTX_TIMELINE
    })
}

fn normalized_events() -> BoxedStrategy<NormalizedEvent> {
    (
        any::<u64>(),
        any::<String>(),
        json_values(),
        any::<String>(),
        prop::option::of(any::<String>()),
        prop::option::of(any::<String>()),
        prop::option::of(any::<String>()),
    )
        .prop_map(
            |(record_id, timestamp, data, evidence_id, event_id, user, command_line)| {
                NormalizedEvent::new(record_id, timestamp, data, evidence_id).with_fields(
                    EventFields {
                        event_id,
                        user,
                        command_line,
                        ..EventFields::default()
                    },
                )
            },
        )
        .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn depth_scanner_matches_serialized_json_structure(value in json_values()) {
        let bytes = serde_json::to_vec(&value).expect("generated JSON serializes");
        let depth = json_depth(&value);
        prop_assert!(!json_depth_exceeds_limit(&bytes, depth));
        if depth > 0 {
            prop_assert!(json_depth_exceeds_limit(&bytes, depth - 1));
        }
    }

    #[test]
    fn normalized_events_round_trip_with_arbitrary_json(event in normalized_events()) {
        let item = event.clone().into_item().expect("event serializes");
        let decoded = NormalizedEvent::from_item(&item).expect("event deserializes");
        prop_assert_eq!(&decoded, &event);
        let expected_reference = NormalizedEvent::raw_reference_for(event.event_record_id);
        prop_assert_eq!(item["raw_reference"].as_str(), Some(expected_reference.as_str()));
    }

    #[test]
    fn logical_references_accept_safe_paths_and_reject_traversal(
        base in logical_references(),
        mutation in 0_u8..7,
    ) {
        prop_assert!(
            validate(&request_json(&base), MAX_LINE_BYTES).is_ok(),
            "safe reference rejected: {base:?}"
        );
        let dangerous = match mutation {
            0 => format!("../{base}"),
            1 => format!("{base}/../child"),
            2 => format!("/{base}"),
            3 => format!("\\{base}"),
            4 => format!("{base}:stream"),
            5 => format!("{base}//child"),
            _ => format!("{base}\0child"),
        };
        prop_assert!(
            validate(&request_json(&dangerous), MAX_LINE_BYTES).is_err(),
            "dangerous reference accepted: {dangerous:?}"
        );
    }
}
