#!/usr/bin/env python3
"""Teste ponta a ponta HEADLESS do pipeline real (F1-34, ADR-0019).

Roda o binário `dfir-app` (subcomandos headless) com o engine real sobre um
EVTX que VOCÊ fornece, num diretório de dados temporário, e confere o contrato
observável de cada etapa. Não usa a GUI e não precisa de display.

    python3 scripts/e2e_headless.py --evtx caminho/para/arquivo.evtx
    python3 scripts/e2e_headless.py --evtx x.evtx --app target/release/dfir-app

Cada etapa imprime PASS/FAIL com o detalhe; o código de saída é 1 se qualquer
etapa falhar. Etapas que dependem de comportamento ainda NÃO corrigido do
backlog aparecem como FAIL de verdade: este script mede o estado real, não o
desejado.

Este script é tooling de QA (AGENTS §2): não é backend do produto e não toca
o banco diretamente.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ElementTree
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RESULTS: list[tuple[str, bool, str]] = []


def record(name: str, ok: bool, detail: str = "") -> bool:
    RESULTS.append((name, ok, detail))
    print(f"[{'PASS' if ok else 'FAIL'}] {name}" + (f" — {detail}" if detail else ""))
    return ok


class Runner:
    def __init__(self, app: Path, engine: Path, data_root: Path, timeout: int):
        self.app, self.timeout = app, timeout
        self.env = dict(os.environ)
        self.env["DFIR_DATA_ROOT"] = str(data_root)
        self.env["DFIR_ENGINE_PATH"] = str(engine)

    def run(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(self.app), *args],
            env=self.env,
            capture_output=True,
            text=True,
            timeout=self.timeout,
            check=False,
        )

    def json(self, *args: str) -> tuple[bool, object, str]:
        proc = self.run(*args)
        if proc.returncode != 0:
            return False, None, (proc.stderr.strip().splitlines() or ["(sem stderr)"])[-1][:300]
        try:
            return True, json.loads(proc.stdout), ""
        except json.JSONDecodeError as error:
            return False, None, f"stdout não é JSON: {error}"


def file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def most_common_event_id(events: list) -> str | None:
    """Event ID mais frequente, para os testes de filtro não dependerem de um
    arquivo específico (o e2e aceita qualquer EVTX real)."""
    ids = [event.get("event_id") for event in events if event.get("event_id")]
    return max(set(ids), key=ids.count) if ids else None


def fetch_all_events(runner: "Runner", job_id: str):
    """Lê a timeline inteira, página a página (como a GUI faz)."""
    events: list = []
    first = None
    offset, total = 0, None
    while total is None or offset < total:
        ok, page, detail = runner.json("results", job_id, "--offset", str(offset), "--limit", "1000")
        if not ok:
            return events, total or 0, first, detail
        first = first or page
        total = page["total"]
        if not page["events"]:
            break
        events.extend(page["events"])
        offset += len(page["events"])
    return events, total or 0, first, ""


DOCUMENTS = [
    "01-relatorio-forense.md",
    "02-vulnerabilidades-attack.md",
    "03-nist-recomendacoes.md",
    "04-hardening.md",
    "05-criptografia.md",
]
SVG_DOCUMENT = "caminho-critico.svg"
EXPORTED_DOCUMENTS = DOCUMENTS + [SVG_DOCUMENT]
# Contrato do incidente de demonstração (src/service/demo.rs, versão bec-demo-1).
DEMO_VERSION = "bec-demo-1"
DEMO_COUNTS = {
    "attack_mappings": 9,
    "critical_path_stages": 6,
    "iocs": 6,
    "findings": 7,
    "root_causes": 3,
    "recommendations": 6,
    "plan_items": 13,
    "source_gaps": 3,
    "nist_gaps": 10,
}
# Valores fictícios (faixas reservadas) que o mascaramento precisa esconder.
DEMO_SENSITIVE = ["203.0.113.45", "cobranca@fornecedor-demo.example", "Ana Exemplo"]
DEMO_TECHNIQUES = ["T1566", "T1078", "T1114", "T1564", "T1059", "T1098", "T1021", "T1567", "T1657"]
_UNESCAPED_PIPE = re.compile(r"(?<!\\)\|")
_SEPARATOR_ROW = re.compile(r"^\|(\s*:?-+:?\s*\|)+$")


def table_problems(text: str) -> list[str]:
    """Confere cada tabela Markdown: separador na 2ª linha e mesmo nº de colunas."""
    problems: list[str] = []
    block: list[str] = []

    def check(rows: list[str]) -> None:
        if len(rows) < 2 or not _SEPARATOR_ROW.match(rows[1].strip()):
            problems.append(f"tabela sem linha separadora: {rows[0][:60]}")
            return
        width = len(_UNESCAPED_PIPE.findall(rows[0]))
        for row in rows[1:]:
            if len(_UNESCAPED_PIPE.findall(row)) != width:
                problems.append(f"colunas diferentes do cabeçalho: {row[:80]}")

    for line in text.splitlines() + [""]:
        if line.startswith("|"):
            block.append(line)
        elif block:
            check(block)
            block = []
    return problems


def generate_documents(runner: "Runner", incident_id: str, out: Path, *extra: str):
    """Gera os entregáveis; devolve (ok, pasta, arquivos do JSON, detalhe)."""
    ok, result, detail = runner.json("deliverables", incident_id, "--out", str(out), *extra)
    if not ok:
        return False, None, [], detail
    return True, Path(result["output_dir"]), result["files"], ""


def read_documents(folder: Path) -> dict[str, str]:
    return {name: (folder / name).read_text(encoding="utf-8") for name in DOCUMENTS}


def detection_review(runner: "Runner", incident_id: str, evidence_id: str) -> None:
    """F1-28: o analista decide sobre os candidatos que as regras geraram no EVTX real."""
    ok, candidates, detail = runner.json("candidates", evidence_id)
    if not record("detecção: candidatos listados", ok and isinstance(candidates, list), detail):
        return
    enough = record("detecção: o EVTX real gerou candidatos", len(candidates) >= 2, f"{len(candidates)} candidato(s)")
    record("detecção: todos nascem PENDENTES", all(c["status"] == "PENDING" for c in candidates))
    record(
        "detecção: cada candidato traz regra, técnica, eventos de origem e limite do método",
        all(
            c["rule_id"] and c["technique_id"] and c["event_refs"] and c["limitations"] and c["rule_set_version"]
            for c in candidates
        ),
    )
    record(
        "detecção: as referências apontam para eventos do arquivo (record:N)",
        all(ref.startswith("record:") for c in candidates for ref in c["event_refs"]),
    )
    record(
        "detecção: a confiança de uma regra automática nunca passa de MEDIUM",
        all(c["confidence"] in ("LOW", "MEDIUM") for c in candidates),
    )

    def findings_count() -> int:
        ok, rows, _ = runner.json("findings", evidence_id)
        return sum(1 for item in (rows or []) if item.get("result_type") == "FINDING") if ok else -1

    before = findings_count()
    record("detecção: candidatos sozinhos não criam achado", before == 0, f"achados da evidência={before}")

    if not enough:
        # O arquivo informado pode não conter nenhuma das sequências que as
        # regras detectam (o e2e não exige um EVTX específico). Isso já foi
        # registrado como FAIL acima; sem candidatos não há decisão a testar,
        # e o script precisa continuar em vez de estourar um IndexError.
        record(
            "detecção: revisão de candidatos executada",
            False,
            "o arquivo não gerou os dois candidatos mínimos exigidos pela revisão",
        )
        return

    target = next((c for c in candidates if c["rule_id"] == "R-PRIV-002"), candidates[0])
    other = next(c for c in candidates if c["id"] != target["id"])
    ok, confirmed, detail = runner.json("candidate-confirm", target["id"], "--incident", incident_id, "--note", "revisado no e2e")
    if record("detecção: confirmar cria mapeamento, achado e IOCs", ok and confirmed["finding_id"] and confirmed["attack_mapping_id"], detail):
        record("detecção: o achado passa a existir", findings_count() == before + 1, f"achados={findings_count()}")
    again = runner.run("candidate-confirm", target["id"], "--incident", incident_id)
    record("detecção: confirmar de novo é recusado (decisão final)", again.returncode != 0)
    no_reason = runner.run("candidate-reject", other["id"], "--reason", "   ")
    record("detecção: rejeitar sem motivo é recusado", no_reason.returncode != 0)
    ok, _, detail = runner.json("candidate-reject", other["id"], "--reason", "serviço de manutenção conhecido")
    record("detecção: rejeitar com motivo funciona", ok, detail)
    ok, counted, _ = runner.json("candidates", evidence_id)
    if ok:
        statuses = [c["status"] for c in counted]
        record(
            "detecção: contagem final confirmado/rejeitado/pendente",
            statuses.count("CONFIRMED") == 1 and statuses.count("REJECTED") == 1 and statuses.count("PENDING") == len(candidates) - 2,
            f"{statuses.count('CONFIRMED')}/{statuses.count('REJECTED')}/{statuses.count('PENDING')}",
        )
    ok, again_run, detail = runner.json("detect", evidence_id)
    if record("detecção: rodar as regras de novo conclui", ok and again_run["status"] == "COMPLETED", detail):
        record(
            "detecção: nova execução não duplica nem desfaz decisões",
            again_run["new_candidates"] == 0 and again_run["counts"]["confirmed"] == 1 and again_run["counts"]["rejected"] == 1,
            json.dumps(again_run["counts"]),
        )


def demo_and_documents(runner: "Runner", evtx: Path, expected_sha: str, data_root: Path) -> None:
    """F1-11/F1-12 (Etapa 7): incidente de demonstração e os cinco documentos."""
    ok, demo, detail = runner.json("demo", str(evtx))
    if not record("demo: comando concluiu (EVTX e engine reais)", ok, detail):
        return
    incident_id, evidence_id = demo["incident_id"], demo["evidence_id"]
    record("demo: versão do conjunto de dados", demo["dataset_version"] == DEMO_VERSION, str(demo["dataset_version"]))
    record("demo: nome do incidente rotulado [DEMO]", str(demo["incident_name"]).startswith("[DEMO]"), str(demo["incident_name"]))
    record("demo: contagens do cenário", demo["counts"] == DEMO_COUNTS, json.dumps(demo["counts"], sort_keys=True))
    record("demo: três análises reais (timeline, YARA e detecção)", len(demo["analysis_job_ids"]) == 3, str(len(demo["analysis_job_ids"])))
    for job_id in demo["analysis_job_ids"]:
        ok, page, detail = runner.json("results", job_id, "--limit", "1")
        record(f"demo: job {job_id[:8]} concluído", ok and page["job"]["status"] == "COMPLETED", detail)

    ok, listing, _ = runner.json("evidence-list")
    row = next((item for item in listing or [] if item["id"] == evidence_id), None)
    record("demo: evidência classificada como DEMO", row is not None and row["status"] == "DEMO", str(row and row["status"]))
    record("demo: SHA-256 da evidência igual ao do arquivo", row is not None and row["sha256"] == expected_sha)

    missing = runner.run("demo", str(data_root / "nao-existe.evtx"))
    record("demo: arquivo inexistente é recusado", missing.returncode != 0)

    # Revisão dos candidatos de detecção (F1-28): confirmar um, rejeitar outro,
    # deixar o resto pendente, e conferir que só a confirmação cria achado.
    detection_review(runner, incident_id, evidence_id)

    # Documentos mascarados (padrão) e sem máscara.
    base = data_root / "docs"
    ok, masked_dir, files, detail = generate_documents(runner, incident_id, base / "mascarado")
    if not record("deliverables: geração mascarada concluiu", ok, detail):
        return
    names = sorted(entry["file_name"] for entry in files)
    record("deliverables: cinco Markdown, SVG e manifest.json", names == sorted(EXPORTED_DOCUMENTS + ["manifest.json"]), ",".join(names))
    reported = {entry["file_name"]: entry for entry in files}
    wrong = [
        name
        for name in EXPORTED_DOCUMENTS + ["manifest.json"]
        if file_digest(masked_dir / name) != reported[name]["sha256"]
        or (masked_dir / name).stat().st_size != reported[name]["bytes"]
    ]
    record("deliverables: SHA-256 e tamanho do JSON batem com os arquivos", not wrong, ",".join(wrong))

    manifest = json.loads((masked_dir / "manifest.json").read_text(encoding="utf-8"))
    by_file = {doc["file"]: doc for doc in manifest["documents"]}
    record("manifest: schema e incidente", manifest["schema"] == "dfir-deliverables/1" and manifest["incident_id"] == incident_id)
    record("manifest: nome do incidente com o rótulo DEMO", str(manifest["incident_name"]).startswith("[DEMO]"))
    record(
        "manifest: hash de cada documento confere",
        sorted(by_file) == sorted(EXPORTED_DOCUMENTS)
        and all(by_file[name]["sha256"] == file_digest(masked_dir / name) for name in EXPORTED_DOCUMENTS),
    )
    record("manifest: declara o mascaramento e o escopo EVTX", manifest["redaction"]["masked"] is True and "EVTX" in manifest["scope"])

    masked_svg = (masked_dir / SVG_DOCUMENT).read_text(encoding="utf-8")
    try:
        svg_root = ElementTree.fromstring(masked_svg)
        svg_valid = svg_root.tag == "{http://www.w3.org/2000/svg}svg"
    except ElementTree.ParseError:
        svg_valid = False
    record(
        "SVG: XML bem formado, alternativa textual e sem script/recurso externo",
        svg_valid and "<desc " in masked_svg and "<script" not in masked_svg and "href=" not in masked_svg,
    )

    docs = read_documents(masked_dir)
    record(
        "documentos: todos rotulados DEMO (título e marca de sintético)",
        all("DEMO" in text.splitlines()[0] and "DEMO (sintético)" in text for text in docs.values()),
    )
    record("documentos: nenhuma seção vazia no cenário completo", all("*Sem registro" not in text for text in docs.values()))
    record("documentos: sem HTML bruto", all(re.search(r"<[A-Za-z/!]", text) is None for text in docs.values()))
    problems = [f"{name}: {issue}" for name, text in docs.items() for issue in table_problems(text)]
    record("documentos: tabelas Markdown bem formadas", not problems, "; ".join(problems[:3]))

    report, attack, nist, hardening, crypto = (docs[name] for name in DOCUMENTS)
    # O relatório escapa caracteres especiais de Markdown (`_` vira `\_`), o
    # que é correto; a conferência compara o nome sem essas barras.
    report_unescaped = report.replace("\\_", "_")
    record(
        "E1: evidência com SHA-256 completo e arquivo de origem",
        expected_sha in report and evtx.name in report_unescaped,
    )
    record("E1: linha do tempo real do EVTX resumida", "Linha do tempo" in report and "sem análise" not in report.lower())
    record("E1: seção de detecção automática com confirmados, rejeitados e pendentes", "4.3 Detecção automática por regras (EVTX)" in report and "Os **pendentes** não foram avaliados" in report)
    detection_row = next(
        (
            line
            for line in report_unescaped.splitlines()
            if evtx.name in line and "2026." in line and line.startswith("|")
        ),
        "",
    )
    cells = [cell.strip() for cell in detection_row.split("|")]
    record("E1: a linha da evidência traz 1 confirmado e 1 rejeitado", len(cells) >= 7 and cells[3] == "1" and cells[4] == "1", detection_row[:160])
    record("E2: as nove técnicas ATT&CK do cenário", all(technique in attack for technique in DEMO_TECHNIQUES))
    gap_section = nist.split("## 3. Gaps vigentes")[-1].split("## 4.")[0]
    gap_ids = re.findall(r"^\| `((?:ID|GV|PR|DE|RS|RC)\.[A-Z]{2}-\d{2})` ", gap_section, re.M)
    record("E3: dez lacunas NIST vigentes, com PR.AA-03 e DE.CM-03", len(gap_ids) == DEMO_COUNTS["nist_gaps"] and {"PR.AA-03", "DE.CM-03"} <= set(gap_ids), ",".join(gap_ids))
    record("E4: cinco grupos de hardening e 9 itens", sum(1 for line in hardening.splitlines() if line.startswith("## ")) >= 5 and sum(1 for line in hardening.splitlines() if line.startswith("### ")) == 9)
    record("E5: três dimensões e 4 itens de proteção", all(f"## {n}." in crypto for n in (1, 2, 3)) and sum(1 for line in crypto.splitlines() if line.startswith("### ")) == 4)
    gap_rows = [line for line in crypto.splitlines() if line.startswith("| Credenciais e segredos")]
    record("E5: a lacuna da matriz de proteção está marcada", len(gap_rows) == 1 and gap_rows[0].count("— (sem item)") == 2)

    masked_text = "\n".join(docs.values()) + "\n" + masked_svg
    leaked = [value for value in DEMO_SENSITIVE if value in masked_text]
    record("máscara: IP, e-mail e nome fictícios não aparecem", not leaked, ",".join(leaked))
    record("máscara: marcadores no lugar dos valores", "{IP-1}" in masked_text and "{EMAIL-1}" in masked_text and "{PESSOA-1}" in masked_text)

    ok, raw_dir, _, detail = generate_documents(runner, incident_id, base / "bruto", "--no-redaction")
    if record("deliverables: geração sem máscara concluiu", ok, detail):
        raw_text = "\n".join(read_documents(raw_dir).values()) + "\n" + (raw_dir / SVG_DOCUMENT).read_text(encoding="utf-8")
        absent = [value for value in DEMO_SENSITIVE if value not in raw_text]
        record("sem máscara: os valores aparecem", not absent, ",".join(absent))
        raw_manifest = json.loads((raw_dir / "manifest.json").read_text(encoding="utf-8"))
        record("sem máscara: o manifest registra a escolha", raw_manifest["redaction"]["masked"] is False)

    # F1-40: chave estritamente sintética e fora do armazenamento interno.
    with tempfile.TemporaryDirectory(prefix="dfir-e2e-keys-") as key_root:
        private_key = Path(key_root) / "synthetic-ed25519.key"
        private_hex = "42" * 32
        private_key.write_text(private_hex + "\n", encoding="ascii")
        ok, signed, detail = runner.json(
            "deliverables", incident_id, "--out", str(base / "assinado"), "--sign-with", str(private_key)
        )
        if record("assinatura: geração assinada concluiu", ok, detail):
            signed_dir = Path(signed["output_dir"])
            public_key = signed["public_key"]
            signature_path = signed_dir / "manifest.json.sig"
            envelope = json.loads(signature_path.read_text(encoding="utf-8"))
            record(
                "assinatura: oito arquivos e digest exato do manifest",
                len(signed["files"]) == 8
                and envelope["schema"] == "dfir-deliverables-signature/1"
                and envelope["algorithm"] == "Ed25519"
                and envelope["manifest_sha256"] == file_digest(signed_dir / "manifest.json"),
            )
            record(
                "assinatura: resposta traz só chave pública/fingerprint",
                len(public_key) == 64
                and signed["public_key_id"].startswith("sha256:")
                and private_hex not in json.dumps(signed)
                and str(private_key) not in json.dumps(signed),
            )
            ok, verified, detail = runner.json(
                "verify-deliverables", str(signed_dir), "--public-key", public_key
            )
            record(
                "assinatura: verificação offline do conjunto",
                ok and verified["documents"] == 6 and verified["public_key_id"] == signed["public_key_id"],
                detail,
            )
            wrong = runner.run("verify-deliverables", str(signed_dir), "--public-key", "43" * 32)
            record("assinatura: chave pública errada é recusada", wrong.returncode != 0)

            manifest_path = signed_dir / "manifest.json"
            original = manifest_path.read_bytes()
            manifest_path.write_bytes(original + b" ")
            changed = runner.run("verify-deliverables", str(signed_dir), "--public-key", public_key)
            record("assinatura: manifest alterado é recusado", changed.returncode != 0)
            manifest_path.write_bytes(original)

            document_path = signed_dir / DOCUMENTS[0]
            original = document_path.read_bytes()
            document_path.write_bytes(original + b"\n")
            changed = runner.run("verify-deliverables", str(signed_dir), "--public-key", public_key)
            record("assinatura: documento alterado é recusado", changed.returncode != 0)
            document_path.write_bytes(original)

            original = signature_path.read_bytes()
            changed_envelope = dict(envelope)
            signature_hex = changed_envelope["signature"]
            changed_envelope["signature"] = ("0" if signature_hex[0] != "0" else "1") + signature_hex[1:]
            signature_path.write_text(json.dumps(changed_envelope), encoding="utf-8")
            changed = runner.run("verify-deliverables", str(signed_dir), "--public-key", public_key)
            record("assinatura: assinatura alterada é recusada", changed.returncode != 0)
            signature_path.write_bytes(original)

            signature_path.rename(signed_dir / "signature-backup")
            missing = runner.run("verify-deliverables", str(signed_dir), "--public-key", public_key)
            record("assinatura: artefato ausente é recusado", missing.returncode != 0)
            (signed_dir / "signature-backup").rename(signature_path)

            ok, unsigned_dir, _, detail = generate_documents(runner, incident_id, base / "assinado")
            record(
                "assinatura: geração não assinada remove assinatura obsoleta",
                ok and not (unsigned_dir / "manifest.json.sig").exists(),
                detail,
            )

    ok, again_dir, _, detail = generate_documents(runner, incident_id, base / "repetido")
    if record("determinismo: segunda geração concluiu", ok, detail):
        differing = [name for name in EXPORTED_DOCUMENTS if file_digest(again_dir / name) != file_digest(masked_dir / name)]
        record("determinismo: documentos idênticos byte a byte", not differing, ",".join(differing))

    verify = runner.run("audit-verify")
    record("audit-verify: íntegra com o cenário e as gerações", verify.returncode == 0, (verify.stderr.strip().splitlines() or [""])[-1][:200])

    # Descarte da evidência da demo: os documentos continuam citando hash e custódia.
    right = runner.run("destroy", evidence_id, "--confirm-filename", evtx.name)
    if not record("demo: descarte da evidência concluiu", right.returncode == 0, (right.stderr.strip().splitlines() or [""])[-1][:200]):
        return
    ok, after_dir, _, detail = generate_documents(runner, incident_id, base / "apos-descarte")
    if record("descarte: documentos ainda são gerados", ok, detail):
        after = read_documents(after_dir)["01-relatorio-forense.md"]
        record("descarte: o relatório não nega a evidência citada", "nenhuma evidência citada" not in after and "Evidências citadas pelo incidente: 1." in after)
        record("descarte: hash, descarte e custódia permanecem", expected_sha in after and "foi descartada localmente" in after and "`DESTROYED`" in after)
    verify = runner.run("audit-verify")
    record("audit-verify após descarte da demo", verify.returncode == 0)


def structured_checks(runner: "Runner", job_id: str, events: list, total: int) -> None:
    """F1-26: campos estruturais do evento e o filtro estruturado da timeline."""
    record(
        "estrutura: eventos trazem host, conta e tipo de fonte",
        any(e.get("host") for e in events) and any(e.get("user") for e in events) and any(e.get("source_type") == "EVTX" for e in events),
    )
    ids = [e.get("event_id") for e in events]
    record("estrutura: Event ID em todos os eventos", all(ids), f"{sum(1 for i in ids if i)}/{len(ids)}")
    sample_id = most_common_event_id(events)
    expected_sample = sum(1 for i in ids if i == sample_id)
    ok, page, detail = runner.json(
        "results", job_id, "--filter", f"event:{sample_id or '4624'}", "--limit", "1"
    )
    record(
        "filtro event:<id presente> devolve exatamente os eventos desse id",
        sample_id is not None and ok and page["total"] == expected_sample > 0,
        f"{page['total'] if ok else detail} vs {expected_sample}",
    )
    # Um segundo ID presente (se houver) prova a semântica de OU da vírgula.
    second_id = next((i for i in ids if i and i != sample_id), sample_id)
    expected_both = sum(1 for i in ids if i in (sample_id, second_id))
    ok, page, _ = runner.json(
        "results", job_id, "--filter", f"event:{sample_id},{second_id}", "--limit", "1"
    )
    record("filtro event:A,B é OU", ok and page["total"] == expected_both, f"{page['total'] if ok else ''} vs {expected_both}")
    user_event = next((e for e in events if e.get("user")), None)
    if user_event:
        user = user_event["user"]
        user_event_id = user_event.get("event_id")
        expected = sum(
            1
            for e in events
            if e.get("user")
            and user.lower() in e["user"].lower()
            and (not user_event_id or e.get("event_id") == user_event_id)
        )
        term = f"event:{user_event_id} user:{user}" if user_event_id else f"user:{user}"
        ok, page, _ = runner.json("results", job_id, "--filter", term, "--limit", "1")
        record("filtro com dois termos é E", ok and page["total"] == expected > 0, f"{page['total'] if ok else ''} vs {expected}")
    ok, page, _ = runner.json("results", job_id, "--filter", "outcome:failure", "--limit", "1")
    expected_fail = sum(1 for e in events if e.get("outcome") == "failure")
    record("filtro outcome:failure confere com os campos", ok and page["total"] == expected_fail, f"{page['total'] if ok else ''} vs {expected_fail}")
    ok, page, _ = runner.json("results", job_id, "--filter", "from:2999-01-01", "--limit", "1")
    record("filtro de data futura não devolve nada", ok and page["total"] == 0)
    bad = runner.run("results", job_id, "--filter", "from:ontem")
    record("filtro com data inválida é erro, não texto livre", bad.returncode != 0 and "invalid date" in (bad.stderr + bad.stdout))


def sandbox_checks(engine: Path, evtx: Path, data_root: Path, evidence: dict) -> None:
    """F1-22: o engine real, contido de verdade, sobre o arquivo real."""
    if sys.platform != "linux":
        record("sandbox: prova de contenção só roda em Linux", True, f"plataforma {sys.platform}")
        return
    stored = (data_root / evidence["storage_reference"]).resolve()
    env = {"DFIR_ENGINE_SANDBOX": "required", "DFIR_ENGINE_INPUT_FILE": str(stored)}
    proc = subprocess.run([str(engine), "--sandbox-selftest"], env=env, capture_output=True, text=True, timeout=60, check=False)
    try:
        report = json.loads(proc.stdout.strip())
    except json.JSONDecodeError:
        record("sandbox: selftest do engine", False, (proc.stdout + proc.stderr)[:200])
        return
    if "sandbox_error" in report:
        record("sandbox: o kernel aplicou Landlock e seccomp", False, report["sandbox_error"][:200])
        return
    probes = report["probes"]
    record("sandbox: todas as camadas aplicadas (no_new_privs, Landlock, seccomp)", report["fully_applied"], report["sandbox"])
    record("sandbox: lê o arquivo de evidência do job", probes["read_pinned_file"] == 0)
    record("sandbox: outro arquivo é negado (EACCES)", probes["read_other_file"] == 13 and probes["list_other_dir"] == 13)
    record("sandbox: criar arquivo é negado", probes["create_file"] == 13)
    record("sandbox: rede e exec são negados (EPERM)", probes["tcp_socket"] == 1 and probes["unix_socket"] == 1 and probes["execve"] == 1 and probes["ptrace"] == 1)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--evtx", required=True, type=Path, help="arquivo .evtx de teste")
    parser.add_argument("--app", type=Path, default=ROOT / "target/debug/dfir-app")
    parser.add_argument("--engine", type=Path, default=None, help="padrão: dfir-engine ao lado do app")
    parser.add_argument("--timeout", type=int, default=300, help="segundos por comando")
    parser.add_argument("--keep", action="store_true", help="não apaga o diretório de dados temporário")
    args = parser.parse_args()

    evtx = args.evtx.resolve()
    app = args.app.resolve()
    engine = (args.engine or app.with_name("dfir-engine" + (".exe" if os.name == "nt" else ""))).resolve()
    for label, path in (("evtx", evtx), ("app", app), ("engine", engine)):
        if not path.is_file():
            print(f"erro: {label} não encontrado: {path}", file=sys.stderr)
            return 2

    data_root = Path(tempfile.mkdtemp(prefix="dfir-e2e-"))
    print(f"dados temporários: {data_root}")
    runner = Runner(app, engine, data_root, args.timeout)
    expected_sha = file_digest(evtx)

    try:
        # 1. importação preserva e calcula hash
        ok, evidence, detail = runner.json("import", str(evtx))
        if not record("import: comando concluiu", ok, detail):
            return 1
        record("import: SHA-256 igual ao do arquivo de origem", evidence["sha256"] == expected_sha, evidence["sha256"][:16])
        record("import: tamanho preservado", evidence["size_bytes"] == evtx.stat().st_size, str(evidence["size_bytes"]))
        record("import: formato detectado como EVTX", str(evidence["format"]).upper() == "EVTX", str(evidence["format"]))
        evidence_id = evidence["id"]

        ok, listing, detail = runner.json("evidence-list")
        record("evidence-list: evidência aparece", ok and any(item["id"] == evidence_id for item in listing), detail)

        # 2. análise EVTX com o engine real
        ok, analysis, detail = runner.json("analyze", evidence_id)
        analyzed = record("analyze EVTX: job COMPLETED", ok and analysis["status"] == "COMPLETED", detail)
        if analyzed:
            record("analyze EVTX: produziu eventos", analysis["item_count"] > 0, f"item_count={analysis['item_count']}")
            record("analyze EVTX: arquivo íntegro não gera avisos do parser", analysis.get("warnings") == [], json.dumps(analysis.get("warnings")))
            events, total, first_page, detail = fetch_all_events(runner, analysis["job_id"])
            if record("results: timeline lida por páginas", first_page is not None, detail):
                stamps = [event["timestamp"] for event in events]
                record("results: timeline não vazia", len(events) > 0, f"{len(events)} eventos")
                record("results: ordem cronológica crescente", stamps == sorted(stamps))
                record(
                    "results: nenhum evento descartado (item_count == total == eventos lidos)",
                    len(events) == analysis["item_count"] == total,
                    f"item_count={analysis['item_count']} total={total} lidos={len(events)}",
                )
                with_event_id = sum(1 for event in events if event.get("event_id"))
                record("results: Event ID extraído", with_event_id > 0, f"{with_event_id}/{len(events)} com event_id")
                sample_id = most_common_event_id(events)
                ok, filtered, detail = runner.json(
                    "results", analysis["job_id"], "--filter", sample_id or "4624", "--limit", "5"
                )
                if record("results: filtro no banco", ok, detail):
                    record(
                        "results: filtro devolve só eventos que casam, com total exato",
                        sample_id is not None
                        and filtered["total"] == sum(1 for event in events if event.get("event_id") == sample_id) > 0
                        and all(event.get("event_id") == sample_id for event in filtered["events"]),
                        f"total filtrado={filtered['total']}",
                    )
            structured_checks(runner, analysis["job_id"], events, total)
            late_cancel = runner.run("cancel-analysis", evidence_id)
            record("cancelamento: CLI recusa pedido sem análise ativa", late_cancel.returncode != 0)
            ok, completed_page, _ = runner.json("results", analysis["job_id"], "--limit", "1")
            record(
                "cancelamento: corrida tardia preserva COMPLETED",
                ok and completed_page["job"]["status"] == "COMPLETED",
            )
        else:
            record("analyze EVTX: detalhe da falha", False, detail or json.dumps(analysis)[:300])

        sandbox_checks(engine, evtx, data_root, evidence)

        # 3. varredura YARA
        ok, scan, detail = runner.json("analyze", evidence_id, "--kind", "YARA_SCAN")
        scanned = record("analyze YARA: job COMPLETED", ok and scan["status"] == "COMPLETED", detail)
        if scanned:
            ok, findings, detail = runner.json("findings", evidence_id)
            record("findings: resultados YARA legíveis", ok, f"{len(findings) if ok else 0} resultado(s) {detail}")
            ok, rescan, _ = runner.json("analyze", evidence_id, "--kind", "YARA_SCAN")
            ok2, findings_again, _ = runner.json("findings", evidence_id)
            if ok and ok2:
                record(
                    "YARA reexecutado não duplica resultados vigentes (F1-19)",
                    len(findings_again) == len(findings),
                    f"antes={len(findings)} depois={len(findings_again)}",
                )

        # 3b. YARA com arquivos que REALMENTE casam (ASCII e UTF-16LE), para
        # medir o `wide` (AUD-09) e a idempotência (AUD-07/F1-19) com dado real.
        text = "mimikatz # sekurlsa::logonpasswords\n"
        fixtures = {
            "yara-ascii.txt": text.encode("ascii"),
            "yara-utf16.txt": text.encode("utf-16-le"),
        }
        for name, content in fixtures.items():
            sample = data_root / name
            sample.write_bytes(content)
            ok, imported, detail = runner.json("import", str(sample))
            if not record(f"yara fixture {name}: import", ok, detail):
                continue
            ok, scan, detail = runner.json("analyze", imported["id"], "--kind", "YARA_SCAN")
            if not record(f"yara fixture {name}: scan COMPLETED", ok and scan["status"] == "COMPLETED", detail):
                continue
            ok, found, _ = runner.json("findings", imported["id"])
            hits = [item for item in (found or []) if "mimikatz" in json.dumps(item)]
            record(f"yara fixture {name}: regra de Mimikatz casou", ok and len(hits) >= 1, f"{len(hits)} resultado(s)")
            ok, _, _ = runner.json("analyze", imported["id"], "--kind", "YARA_SCAN")
            ok2, found_again, _ = runner.json("findings", imported["id"])
            if ok and ok2:
                again = [item for item in found_again if "mimikatz" in json.dumps(item)]
                record(
                    f"yara fixture {name}: reexecução não duplica o resultado vigente",
                    len(again) == len(hits),
                    f"antes={len(hits)} depois={len(again)}",
                )

        # 3c. EVTX truncado (AUD-36): deve concluir, mas COM aviso explícito.
        # A fração precisa ficar abaixo do tamanho real: `max(70_000, ...)`
        # devolvia o arquivo inteiro para um EVTX menor que 70 KB e o teste
        # passava sem testar truncamento nenhum.
        truncated = data_root / "truncado.evtx"
        size = evtx.stat().st_size
        keep = max(4_096 + 1_024, int(size * 0.55))
        if keep >= size:
            keep = max(4_096 + 1_024, size - max(1_024, size // 10))
        truncated.write_bytes(evtx.read_bytes()[:keep])
        ok, imported, detail = runner.json("import", str(truncated))
        if record("evtx truncado: import", ok, detail):
            ok, scan, detail = runner.json("analyze", imported["id"])
            if record("evtx truncado: job concluiu", ok and scan["status"] == "COMPLETED", detail):
                codes = {warning["code"] for warning in scan["warnings"]}
                record("evtx truncado: aviso EVTX_TRUNCATED presente", "EVTX_TRUNCATED" in codes, ",".join(sorted(codes)))
                ok, page, _ = runner.json("results", scan["job_id"], "--limit", "1")
                record(
                    "evtx truncado: o aviso fica gravado no job",
                    ok and page["job"]["warning_count"] >= 1,
                    str(page["job"]["warning_count"]) if ok else "",
                )

        # 3d. guarda de espaço em disco (ADR-0021): com uma margem absurda, a
        # importação e a análise precisam recusar com mensagem acionável
        strict = Runner(app, engine, data_root, args.timeout)
        strict.env["DFIR_MIN_FREE_DISK_BYTES"] = str(1 << 40)
        refused = strict.run("analyze", evidence_id)
        record(
            "disco: análise recusada quando falta margem de espaço livre",
            refused.returncode != 0 and "not enough free disk space" in (refused.stderr + refused.stdout),
        )
        refused = strict.run("import", str(evtx))
        record(
            "disco: importação recusada quando falta margem de espaço livre",
            refused.returncode != 0 and "not enough free disk space" in (refused.stderr + refused.stdout),
        )

        # 3e. incidente de demonstração e os cinco documentos (F1-11, F1-12)
        demo_and_documents(runner, evtx, expected_sha, data_root)

        # 4. integridade das cadeias
        verify = runner.run("audit-verify")
        record("audit-verify: cadeia de auditoria e custódia íntegras", verify.returncode == 0, (verify.stderr.strip().splitlines() or [""])[-1][:200])

        # 5. descarte exige confirmação exata e preserva auditoria
        wrong = runner.run("destroy", evidence_id, "--confirm-filename", "nome-errado.evtx")
        record("destroy: recusa nome de arquivo incorreto", wrong.returncode != 0)
        stored = data_root / evidence["storage_reference"]
        right = runner.run("destroy", evidence_id, "--confirm-filename", evtx.name)
        record("destroy: concluiu com o nome correto", right.returncode == 0, (right.stderr.strip().splitlines() or [""])[-1][:200])
        record("destroy: arquivo físico removido", not stored.exists(), str(stored.name))
        ok, listing, _ = runner.json("evidence-list")
        record("destroy: evidência some do catálogo", ok and all(item["id"] != evidence_id for item in listing))
        verify = runner.run("audit-verify")
        record("audit-verify após descarte", verify.returncode == 0)
    finally:
        if args.keep:
            print(f"dados mantidos em {data_root}")
        else:
            shutil.rmtree(data_root, ignore_errors=True)

    failed = [name for name, ok, _ in RESULTS if not ok]
    print(f"\n{len(RESULTS) - len(failed)}/{len(RESULTS)} etapas passaram")
    for name in failed:
        print(f"  FALHOU: {name}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
