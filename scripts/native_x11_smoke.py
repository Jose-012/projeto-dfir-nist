#!/usr/bin/env python3
"""Smoke X11 do fluxo nativo, sem screenshots nem AT-SPI.

As ações são injetadas por XTEST, mas o aceite usa pós-condições independentes:
mudança real de tela no log sanitizado, janela Zenity por título/PID, evidência e
jobs no SQLite isolado e página de timeline carregada pelo backend da UI.

O data-root precisa estar vazio. Nenhum arquivo, evento ou nome de evidência é
impresso. Somente IDs, contagens, status e o hash da fixture entram no relatório.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sqlite3
import subprocess
import sys
import time
from pathlib import Path
from typing import NamedTuple

from Xlib import X, XK, display, protocol
from Xlib.ext import xtest

D = display.Display(os.environ.get("DISPLAY", ":0"))
ROOT = D.screen().root
DIALOG_TITLE = "Selecionar arquivo de evidência"


class WindowInfo(NamedTuple):
    window: object
    window_id: int
    name: str
    pid: int | None


def log(message: str) -> None:
    print(f"[{time.strftime('%H:%M:%S')}] {message}", flush=True)


def window_name(window) -> str:
    try:
        prop = window.get_full_property(D.intern_atom("_NET_WM_NAME"), X.AnyPropertyType)
        if prop is not None:
            value = prop.value
            if isinstance(value, bytes):
                return value.decode("utf-8", errors="replace")
            return str(value)
        return str(window.get_wm_name() or "")
    except Exception:
        return ""


def window_pid(window) -> int | None:
    try:
        prop = window.get_full_property(D.intern_atom("_NET_WM_PID"), X.AnyPropertyType)
        if prop is None or not prop.value:
            return None
        return int(prop.value[0])
    except Exception:
        return None


def windows() -> list[WindowInfo]:
    found: list[WindowInfo] = []

    def walk(parent, depth: int) -> None:
        if depth > 16:
            return
        try:
            children = parent.query_tree().children
        except Exception:
            return
        for child in children:
            name = window_name(child)
            if name:
                found.append(WindowInfo(child, child.id, name, window_pid(child)))
            walk(child, depth + 1)

    walk(ROOT, 0)
    return found


def find_dfir() -> WindowInfo | None:
    return next((item for item in windows() if "DFIR" in item.name), None)


def wait_for(predicate, timeout: float, description: str):
    deadline = time.monotonic() + timeout
    last_error = None
    while time.monotonic() < deadline:
        try:
            value = predicate()
            if value:
                return value
        except (sqlite3.Error, OSError) as error:
            last_error = error
        time.sleep(0.2)
    suffix = f"; último erro={type(last_error).__name__}" if last_error else ""
    raise RuntimeError(f"timeout em {description}{suffix}")


def focus(window) -> None:
    window.configure(stack_mode=X.Above)
    active_event = protocol.event.ClientMessage(
        window=window,
        client_type=D.intern_atom("_NET_ACTIVE_WINDOW"),
        data=(32, [1, X.CurrentTime, 0, 0, 0]),
    )
    ROOT.send_event(
        active_event,
        event_mask=X.SubstructureRedirectMask | X.SubstructureNotifyMask,
    )
    window.set_input_focus(X.RevertToParent, X.CurrentTime)
    D.sync()
    time.sleep(0.25)
    focused = D.get_input_focus().focus
    log(f"focus target={window.id} actual={focused.id}")


def absolute_origin(window) -> tuple[int, int]:
    x = y = 0
    current = window
    for _ in range(32):
        if current.id == ROOT.id:
            return x, y
        geometry = current.get_geometry()
        x += geometry.x
        y += geometry.y
        current = current.query_tree().parent
    raise RuntimeError("window ancestry is too deep")


def click(window, x: int, y: int) -> None:
    origin_x, origin_y = absolute_origin(window)
    xtest.fake_input(D, X.MotionNotify, x=origin_x + x, y=origin_y + y)
    D.sync()
    time.sleep(0.08)
    xtest.fake_input(D, X.ButtonPress, 1)
    D.sync()
    time.sleep(0.06)
    xtest.fake_input(D, X.ButtonRelease, 1)
    D.sync()
    time.sleep(0.35)


def key(name: str, presses: int = 1) -> None:
    keycode = D.keysym_to_keycode(XK.string_to_keysym(name))
    if not keycode:
        raise RuntimeError(f"unknown key: {name}")
    for _ in range(presses):
        xtest.fake_input(D, X.KeyPress, keycode)
        xtest.fake_input(D, X.KeyRelease, keycode)
        D.sync()
        time.sleep(0.1)


def ctrl_key(letter: str) -> None:
    control = D.keysym_to_keycode(XK.string_to_keysym("Control_L"))
    keycode = D.keysym_to_keycode(XK.string_to_keysym(letter))
    if not control or not keycode:
        raise RuntimeError(f"Ctrl+{letter} indisponível no mapa de teclado")
    xtest.fake_input(D, X.KeyPress, control)
    D.sync()
    time.sleep(0.05)
    xtest.fake_input(D, X.KeyPress, keycode)
    D.sync()
    time.sleep(0.03)
    xtest.fake_input(D, X.KeyRelease, keycode)
    D.sync()
    time.sleep(0.03)
    xtest.fake_input(D, X.KeyRelease, control)
    D.sync()
    time.sleep(0.1)


def paste_text(text: str) -> None:
    process = subprocess.Popen(
        ["xclip", "-selection", "clipboard", "-quiet", "-loops", "1"],
        stdin=subprocess.PIPE,
        text=True,
    )
    try:
        with process.stdin as source:
            source.write(text)
        ctrl_key("v")
        return_code = process.wait(timeout=10)
        if return_code != 0:
            raise RuntimeError(f"xclip terminou com exit {return_code}")
    finally:
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)


def read_log(path: Path, offset: int = 0) -> tuple[str, int]:
    try:
        with path.open("rb") as handle:
            handle.seek(offset)
            data = handle.read()
            return data.decode("utf-8", errors="replace"), handle.tell()
    except FileNotFoundError:
        return "", offset


def wait_log(
    path: Path,
    needle: str,
    timeout: float,
    offset: int = 0,
    detail: str = "",
) -> int:
    def check():
        text, position = read_log(path, offset)
        for line in text.splitlines():
            if needle in line and (not detail or detail in line):
                return position
        return None

    return wait_for(check, timeout, f"log {needle} {detail}".strip())


def db_rows(db_path: Path, sql: str, params: tuple = ()) -> list[tuple]:
    uri = f"file:{db_path}?mode=ro"
    with sqlite3.connect(uri, uri=True, timeout=0.25) as connection:
        return list(connection.execute(sql, params))


def wait_job(db_path: Path, kind: str, timeout: float) -> tuple:
    def check():
        rows = db_rows(
            db_path,
            "SELECT id,status,item_count FROM analysis_jobs WHERE kind=? ORDER BY created_at DESC LIMIT 1",
            (kind,),
        )
        if rows and rows[0][1] == "COMPLETED":
            return rows[0]
        return None

    return wait_for(check, timeout, f"job {kind} COMPLETED")


def process_exists(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def open_evidence_dialog(app_window: WindowInfo, args) -> WindowInfo:
    click(app_window.window, args.import_x, args.import_y)
    dialog = wait_for(
        lambda: next((item for item in windows() if item.name == DIALOG_TITLE), None),
        args.timeout,
        "janela Zenity com título esperado",
    )
    wait_for(
        lambda: dialog.window.get_attributes().map_state == X.IsViewable,
        5.0,
        "seletor visível",
    )
    return dialog


def open_timeline(
    app_window: WindowInfo,
    log_path: Path,
    x: int,
    preferred_y: int,
) -> tuple[int, int]:
    # De baixo para cima, nenhum cabeçalho acima do alvo é fechado antes dele.
    candidates = sorted(
        {750, 700, preferred_y, 600, 550, 500, 450, 400, 350, 300},
        reverse=True,
    )
    for scroll_attempt in range(3):
        for y in candidates:
            _, offset = read_log(log_path)
            click(app_window.window, x, y)
            time.sleep(0.6)
            text, _ = read_log(log_path, offset)
            for line in text.splitlines():
                try:
                    fields = json.loads(line).get("fields", {})
                except json.JSONDecodeError:
                    continue
                event_count = fields.get("event_count")
                if (
                    fields.get("message") == "native UI timeline page loaded"
                    and fields.get("has_page") is True
                    and isinstance(event_count, int)
                    and event_count > 0
                ):
                    return y, event_count
        if scroll_attempt < 2:
            focus(app_window.window)
            origin_x, origin_y = absolute_origin(app_window.window)
            for _ in range(8):
                xtest.fake_input(
                    D,
                    X.MotionNotify,
                    x=origin_x + 800,
                    y=origin_y + 600,
                )
                xtest.fake_input(D, X.ButtonPress, 5)
                D.sync()
                xtest.fake_input(D, X.ButtonRelease, 5)
                D.sync()
                time.sleep(0.03)
            time.sleep(0.6)
    raise RuntimeError("nenhum ponto visível abriu uma página não vazia da timeline")


def scroll_to_top(app_window: WindowInfo) -> None:
    focus(app_window.window)
    origin_x, origin_y = absolute_origin(app_window.window)
    for _ in range(32):
        xtest.fake_input(D, X.MotionNotify, x=origin_x + 800, y=origin_y + 600)
        xtest.fake_input(D, X.ButtonPress, 4)
        D.sync()
        xtest.fake_input(D, X.ButtonRelease, 4)
        D.sync()
        time.sleep(0.03)
    time.sleep(0.8)


def close_owned_dialogs(owner_pid: int) -> None:
    """Fecha apenas diálogos no process-group isolado do app do smoke."""
    for item in windows():
        if item.name != DIALOG_TITLE or item.pid is None:
            continue
        try:
            if os.getpgid(item.pid) == owner_pid:
                os.kill(item.pid, 15)
        except ProcessLookupError:
            pass
        except PermissionError as error:
            log(f"aviso de cleanup: {type(error).__name__} pid={item.pid}")


def launch(app: Path, data_root: Path, output_path: Path) -> subprocess.Popen:
    env = os.environ.copy()
    env["DISPLAY"] = D.get_display_name()
    env["DFIR_DATA_ROOT"] = str(data_root)
    env["DFIR_FILE_DIALOG_BACKEND"] = "zenity"
    env["RUST_LOG"] = "info"
    output = output_path.open("w", encoding="utf-8")
    process = subprocess.Popen(
        [str(app)],
        env=env,
        stdout=output,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    output.close()
    return process


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", default="target/debug/dfir-app")
    parser.add_argument("--fixture", default="tests/fixtures/senai-demo.evtx")
    parser.add_argument("--data-root", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--timeout", type=float, default=45.0)
    parser.add_argument("--import-method", choices=["manual", "zenity"], default="manual")
    parser.add_argument("--evidence-y", type=int, default=80)
    parser.add_argument("--incidents-y", type=int, default=120)
    parser.add_argument("--import-x", type=int, default=443)
    parser.add_argument("--import-y", type=int, default=77)
    parser.add_argument("--path-x", type=int, default=550)
    parser.add_argument("--path-y", type=int, default=120)
    parser.add_argument("--row-x", type=int, default=258)
    parser.add_argument("--row-y", type=int, default=200)
    parser.add_argument("--analyze-x", type=int, default=960)
    parser.add_argument("--analyze-y", type=int, default=200)
    parser.add_argument("--detect-x", type=int, default=1050)
    parser.add_argument("--detect-y", type=int, default=200)
    parser.add_argument("--timeline-x", type=int, default=275)
    parser.add_argument("--timeline-y", type=int, default=650)
    args = parser.parse_args()

    app = Path(args.app).resolve()
    fixture = Path(args.fixture).resolve()
    data_root = Path(args.data_root).resolve()
    results_path = Path(args.results).resolve()
    results_path.parent.mkdir(parents=True, exist_ok=True)
    results: dict[str, dict] = {}
    proc = None
    dialog_pid = None

    def record(name: str, status: str, **details) -> None:
        results[name] = {"status": status, **details}
        log(f"{name}: {status} {details}")
        results_path.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")

    try:
        if not app.is_file():
            raise RuntimeError("binário não encontrado")
        if not fixture.is_file():
            raise RuntimeError("fixture não encontrada")
        if find_dfir() is not None:
            raise RuntimeError("já existe janela DFIR; o smoke não interage com outra sessão")
        if any(item.name == DIALOG_TITLE for item in windows()):
            raise RuntimeError("já existe seletor de evidência; o smoke não interage com outra sessão")
        if data_root.exists() and any(data_root.iterdir()):
            raise RuntimeError("data-root precisa estar vazio; nenhum dado será removido")
        data_root.mkdir(parents=True, exist_ok=True)
        db_path = data_root / "db" / "dfir.sqlite"
        log_path = data_root / "logs" / "dfir.log"
        fixture_hash = hashlib.sha256(fixture.read_bytes()).hexdigest()
        record("fixture", "PASS", sha256=fixture_hash, size_bytes=fixture.stat().st_size)

        proc = launch(app, data_root, results_path.with_suffix(".app-output.log"))
        app_window = wait_for(find_dfir, min(args.timeout, 20.0), "janela DFIR")
        wait_for(
            lambda: app_window.window.get_attributes().map_state == X.IsViewable,
            5.0,
            "janela DFIR visível",
        )
        wait_log(log_path, "native UI GPU adapter", min(args.timeout, 20.0))
        focus(app_window.window)
        geometry = app_window.window.get_geometry()
        origin_x, origin_y = absolute_origin(app_window.window)
        record(
            "window",
            "PASS",
            window_id=app_window.window_id,
            width=geometry.width,
            height=geometry.height,
            origin_x=origin_x,
            origin_y=origin_y,
        )

        click(app_window.window, 80, args.incidents_y)
        wait_log(log_path, "native UI screen changed", 8.0, detail="Incidents")
        record("navigate_to_incidents", "PASS")
        click(app_window.window, 80, args.evidence_y)
        wait_log(log_path, "native UI screen changed", 8.0, detail="Evidence")
        record("navigate_to_evidence", "PASS")

        import_method = "manual_path"
        if args.import_method == "zenity":
            import_method = "zenity_ctrl_l"
            _, dialog_log_offset = read_log(log_path)
            import_dialog = open_evidence_dialog(app_window, args)
            focus(import_dialog.window)
            ctrl_key("l")
            time.sleep(0.3)
            paste_text(str(fixture))
            key("Return")
            wait_log(
                log_path,
                "file dialog finished",
                args.timeout,
                offset=dialog_log_offset,
                detail='"outcome":"path"',
            )
            wait_for(
                lambda: import_dialog.window_id not in {item.window_id for item in windows()},
                8.0,
                "fechamento do seletor após escolha",
            )
        else:
            focus(app_window.window)
            click(app_window.window, args.path_x, args.path_y)
            paste_text(str(fixture))
            key("Tab")
            key("space")
        imported = wait_for(
            lambda: next(
                iter(db_rows(db_path, "SELECT id,format,size_bytes,status FROM evidence")),
                None,
            ),
            args.timeout,
            "evidência importada no catálogo SQLite",
        )
        if imported[1] != "evtx" or imported[2] != fixture.stat().st_size:
            raise RuntimeError("a evidência gravada não corresponde à fixture")
        record(
            "import_fixture",
            "PASS",
            method=import_method,
            evidence_id=imported[0],
            format=imported[1],
            size_bytes=imported[2],
            evidence_status=imported[3],
        )
        if args.import_method == "zenity":
            focus(app_window.window)
        time.sleep(0.8)

        click(app_window.window, args.analyze_x, args.analyze_y)
        timeline_job = wait_job(db_path, "EVTX_TIMELINE", args.timeout)
        if timeline_job[2] <= 0:
            raise RuntimeError("análise EVTX concluiu sem eventos indexados")
        timeline_count = wait_for(
            lambda: next(
                iter(
                    db_rows(
                        db_path,
                        "SELECT event_count FROM timeline_index WHERE job_id=?",
                        (timeline_job[0],),
                    )
                ),
                None,
            ),
            args.timeout,
            "timeline_index com contagem",
        )[0]
        if timeline_count <= 0:
            raise RuntimeError("timeline_index vazia após análise")
        record(
            "analyze_evtx",
            "PASS",
            job_id=timeline_job[0],
            item_count=timeline_job[2],
            timeline_events=timeline_count,
        )
        time.sleep(0.8)

        click(app_window.window, args.detect_x, args.detect_y)
        detection_job = wait_job(db_path, "EVTX_DETECTION", args.timeout)
        if detection_job[2] <= 0:
            raise RuntimeError("detecção concluiu sem candidatos para a fixture")
        record("detect", "PASS", job_id=detection_job[0], item_count=detection_job[2])
        time.sleep(0.8)

        click(app_window.window, args.row_x, args.row_y)
        time.sleep(1.0)
        timeline_y, timeline_page_events = open_timeline(
            app_window,
            log_path,
            args.timeline_x,
            args.timeline_y,
        )
        record(
            "timeline_view",
            "PASS",
            job_id=timeline_job[0],
            timeline_index_events=timeline_count,
            timeline_page_events=timeline_page_events,
            timeline_y=timeline_y,
        )

        scroll_to_top(app_window)
        dialog = open_evidence_dialog(app_window, args)
        dialog_pid = dialog.pid
        focus(dialog.window)
        key("Escape")
        wait_for(
            lambda: dialog.window_id not in {item.window_id for item in windows()},
            8.0,
            "cancelamento do seletor",
        )
        if dialog_pid is not None:
            wait_for(lambda: not process_exists(dialog_pid), 8.0, "encerramento do processo seletor")
        if db_rows(db_path, "SELECT COUNT(*) FROM evidence") != [(1,)]:
            raise RuntimeError("cancelar o seletor alterou o catálogo de evidências")
        record("selector_open_cancel", "PASS", dialog_pid=dialog_pid, evidence_count=1)
        dialog_pid = None
        record("flow", "PASS")

    except Exception as error:
        record("flow", "FAIL", error_type=type(error).__name__, error=str(error))
        return 1
    finally:
        if proc is not None and proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=6)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=3)
        if proc is not None:
            close_owned_dialogs(proc.pid)
        results_path.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    sys.exit(main())
