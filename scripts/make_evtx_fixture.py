#!/usr/bin/env python3
"""Gera a fixture EVTX sanitizada e determinística do projeto (F1-16).

O projeto não distribui `.evtx` de terceiros: esta fixture é *construída* aqui,
byte a byte, a partir de eventos sintéticos. Ela é determinística (mesma saída
para a mesma versão deste script) e o SHA-256 fica registrado em
`tests/fixtures/README.md`.

Uso:
    python3 scripts/make_evtx_fixture.py [saida.evtx]

Formato (EVTX 3.1, um chunk): cabeçalho de arquivo de 4096 bytes, um chunk de
64 KiB com cabeçalho de 512 bytes, tabela de strings a partir do offset 512 e
registros BinXML sem template (tokens diretos), aceitos pelo crate `evtx`
0.15.1 usado pelo engine.
"""

from __future__ import annotations

import struct
import sys
import zlib
from dataclasses import dataclass, field
from pathlib import Path

FILE_HEADER_SIZE = 4096
CHUNK_SIZE = 65536
CHUNK_HEADER_SIZE = 512

# Tokens BinXML
TOKEN_EOF = 0x00
TOKEN_OPEN_START = 0x01
TOKEN_OPEN_START_ATTRS = 0x41
TOKEN_CLOSE_START = 0x02
TOKEN_CLOSE_EMPTY = 0x03
TOKEN_CLOSE_ELEMENT = 0x04
TOKEN_VALUE = 0x05
TOKEN_VALUE_MORE = 0x45
TOKEN_ATTRIBUTE = 0x06
TOKEN_ATTRIBUTE_MORE = 0x46
TOKEN_FRAGMENT_HEADER = 0x0F

TYPE_STRING = 0x01
TYPE_UINT16 = 0x06

# 2026-11-01T09:00:00Z em FILETIME (100 ns desde 1601-01-01).
FILETIME_EPOCH_2026_11_01T09Z = 133_845_636_000_000_000


@dataclass
class Element:
    name: str
    attributes: list[tuple[str, str, int]] = field(default_factory=list)
    children: list[Element] = field(default_factory=list)
    text: str | None = None
    text_type: int = TYPE_STRING


def escape(value: str) -> str:
    return value.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


class NameTable:
    """Tabela de strings do chunk: nome -> offset absoluto no chunk."""

    def __init__(self, start: int = CHUNK_HEADER_SIZE) -> None:
        self._start = start
        self._offsets: dict[str, int] = {}
        self._blob = bytearray()
        self._entry_offsets: list[int] = []
        self._entry_lengths: list[int] = []

    def offset(self, name: str) -> int:
        if name in self._offsets:
            return self._offsets[name]
        offset = self._start + len(self._blob)
        entry = bytearray()
        entry += struct.pack("<IH", 0, 0)  # next_string, hash
        entries = name.encode("utf-16-le")
        entry += struct.pack("<H", len(entries) // 2)
        entry += entries
        entry += struct.pack("<H", 0)
        self._entry_offsets.append(offset)
        self._entry_lengths.append(len(entry))
        self._blob += entry
        self._offsets[name] = offset
        return offset

    @property
    def blob(self) -> bytes:
        """Entradas encadeadas por `next_string` (o cache do parser segue a cadeia)."""
        blob = bytearray(self._blob)
        for index in range(len(self._entry_offsets) - 1):
            start = self._entry_offsets[index] - self._start
            blob[start:start + 4] = struct.pack("<I", self._entry_offsets[index + 1])
        return bytes(blob)


def encode_value_type(value: str, value_type: int) -> bytes:
    if value_type == TYPE_UINT16:
        return bytes([TYPE_UINT16]) + struct.pack("<H", int(value))
    data = value.encode("utf-16-le")
    return bytes([TYPE_STRING]) + struct.pack("<H", len(data) // 2) + data


def encode_element(element: Element, names: NameTable) -> bytes:
    """Serializa um elemento BinXML (tokens diretos, sem template)."""
    out = bytearray()

    if element.attributes:
        body = bytearray()
        attribute_list = bytearray()
        for index, (attr_name, attr_value, attr_type) in enumerate(element.attributes):
            is_last = index == len(element.attributes) - 1
            attribute_list += bytes(
                [TOKEN_ATTRIBUTE if is_last else TOKEN_ATTRIBUTE_MORE]
            )
            attribute_list += struct.pack("<I", names.offset(attr_name))
            attribute_list += bytes([TOKEN_VALUE if is_last else TOKEN_VALUE_MORE])
            attribute_list += encode_value_type(attr_value, attr_type)
        body += struct.pack("<I", len(attribute_list))
        body += attribute_list
        head = bytes([TOKEN_OPEN_START_ATTRS]) + struct.pack("<I", 0)
        head += struct.pack("<I", names.offset(element.name))
        out += head
        out += struct.pack("<I", len(attribute_list))
        out += attribute_list
        if not element.children and element.text is None:
            out += bytes([TOKEN_CLOSE_EMPTY])
            return bytes(out)
        out += bytes([TOKEN_CLOSE_START])
    else:
        out += bytes([TOKEN_OPEN_START]) + struct.pack("<I", 0)
        out += struct.pack("<I", names.offset(element.name))
        if not element.children and element.text is None:
            out += bytes([TOKEN_CLOSE_EMPTY])
            return bytes(out)
        out += bytes([TOKEN_CLOSE_START])

    if element.text is not None:
        out += bytes([TOKEN_VALUE]) + encode_value_type(element.text, element.text_type)

    for child in element.children:
        out += encode_element(child, names)

    out += bytes([TOKEN_CLOSE_ELEMENT])
    return bytes(out)


def encode_record_binxml(root: Element, names: NameTable) -> bytes:
    fragment = bytes([TOKEN_FRAGMENT_HEADER, 0x01, 0x01, 0x00])
    return fragment + encode_element(root, names) + bytes([TOKEN_EOF])


def data_element(name: str, value: str) -> Element:
    return Element("Data", attributes=[("Name", name, TYPE_STRING)], text=value)


def event(
    event_id: int,
    timestamp_index: int,
    computer: str,
    provider: str,
    data: list[tuple[str, str]],
) -> Element:
    # Timestamp determinístico: base 09:00:00Z + timestamp_index segundos.
    hours, rest = divmod(timestamp_index, 3600)
    minutes, seconds = divmod(rest, 60)
    milliseconds = (timestamp_index * 137) % 1000
    system_time = (
        f"2026-11-01T{9 + hours:02d}:{minutes:02d}:{seconds:02d}.{milliseconds:03d}0000Z"
    )
    security = provider == "Microsoft-Windows-Security-Auditing"
    system = Element(
        "System",
        children=[
            Element("Provider", attributes=[("Name", provider, TYPE_STRING)]),
            Element("EventID", text=str(event_id), text_type=TYPE_UINT16),
            Element("TimeCreated", attributes=[("SystemTime", system_time, TYPE_STRING)]),
            Element("Channel", text="Security" if security else "System"),
            Element("Computer", text=computer),
        ],
    )
    event_data = Element("EventData", children=[data_element(k, v) for k, v in data])
    return Element("Event", children=[system, event_data])


SECURITY = "Microsoft-Windows-Security-Auditing"
SERVICE_CONTROL = "Service Control Manager"

# Eventos sintéticos: nenhum dado pessoal real, host e contas fictícios.
EVENTS: list[Element] = [
    event(4624, 0, "srv-demo", SECURITY, [
        ("TargetUserName", "ana"),
        ("TargetDomainName", "SENAI"),
        ("LogonType", "3"),
        ("IpAddress", "10.0.0.10"),
    ]),
    event(4625, 30, "srv-demo", SECURITY, [
        ("TargetUserName", "joao"),
        ("TargetDomainName", "SENAI"),
        ("LogonType", "3"),
        ("IpAddress", "10.0.0.66"),
        ("Status", "0xc000006d"),
    ]),
    event(4625, 34, "srv-demo", SECURITY, [
        ("TargetUserName", "joao"),
        ("TargetDomainName", "SENAI"),
        ("LogonType", "3"),
        ("IpAddress", "10.0.0.66"),
        ("Status", "0xc000006d"),
    ]),
    event(4625, 38, "srv-demo", SECURITY, [
        ("TargetUserName", "joao"),
        ("TargetDomainName", "SENAI"),
        ("LogonType", "3"),
        ("IpAddress", "10.0.0.66"),
        ("Status", "0xc000006d"),
    ]),
    event(4625, 42, "srv-demo", SECURITY, [
        ("TargetUserName", "joao"),
        ("TargetDomainName", "SENAI"),
        ("LogonType", "3"),
        ("IpAddress", "10.0.0.66"),
        ("Status", "0xc000006d"),
    ]),
    event(4625, 46, "srv-demo", SECURITY, [
        ("TargetUserName", "joao"),
        ("TargetDomainName", "SENAI"),
        ("LogonType", "3"),
        ("IpAddress", "10.0.0.66"),
        ("Status", "0xc000006d"),
    ]),
    event(4672, 60, "srv-demo", SECURITY, [
        ("SubjectUserName", "ana"),
        ("SubjectDomainName", "SENAI"),
        ("PrivilegeList", "SeDebugPrivilege SeTcbPrivilege SeLoadDriverPrivilege"),
    ]),
    event(4688, 90, "srv-demo", SECURITY, [
        ("SubjectUserName", "ana"),
        ("NewProcessName", "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
        ("CommandLine", "powershell.exe -nop -w hidden -EncodedCommand SQBFAFgA"),
        ("ParentProcessName", "C:\\Windows\\explorer.exe"),
    ]),
    event(4720, 120, "srv-demo", SECURITY, [
        ("TargetUserName", "tmp-backup"),
        ("SubjectUserName", "ana"),
        ("SubjectDomainName", "SENAI"),
    ]),
    event(4732, 150, "srv-demo", SECURITY, [
        ("MemberName", "CN=tmp-backup,CN=Users,DC=senai,DC=local"),
        ("TargetUserName", "Administrators"),
        ("SubjectUserName", "ana"),
    ]),
    event(7045, 180, "srv-demo", SERVICE_CONTROL, [
        ("ServiceName", "DemoSvc"),
        ("ImagePath", "C:\\Windows\\Temp\\demo.exe"),
        ("ServiceType", "user mode service"),
        ("StartType", "demand start"),
    ]),
    event(1102, 210, "srv-demo", SECURITY, [
        ("SubjectUserName", "ana"),
        ("SubjectDomainName", "SENAI"),
    ]),
]


def build_chunk() -> tuple[bytes, int]:
    """Layout real do chunk: registros a partir de 512 e tabela de strings depois.

    O parser (`evtx` 0.15.1) começa a caminhada de registros em
    `EVTX_CHUNK_HEADER_SIZE` (512) e resolve nomes pelo cache montado a partir
    dos offsets do cabeçalho; por isso a tabela de strings fica **após** o
    último registro, dentro da área viva (`free_space_offset`).
    """
    # Primeira passada: nomes com offsets relativos, só para medir os registros e
    # a tabela. Os offsets reais da tabela vêm depois (ela fica após os registros).
    measure = NameTable(start=0)
    sizes = [len(encode_record_binxml(element, measure)) for element in EVENTS]
    # Cada registro custa 24 bytes de cabeçalho + BinXML + 4 bytes do tamanho final.
    string_table_offset = CHUNK_HEADER_SIZE + sum(size + 28 for size in sizes)
    names = NameTable(start=string_table_offset)
    encoded = [encode_record_binxml(element, names) for element in EVENTS]

    chunk = bytearray(CHUNK_SIZE)
    chunk[0:8] = b"ElfChnk\x00"

    offset = CHUNK_HEADER_SIZE
    offsets: list[int] = []
    for index, binxml in enumerate(encoded):
        offsets.append(offset)
        record_id = index + 1
        size = 24 + len(binxml) + 4
        chunk[offset:offset + 4] = b"\x2a\x2a\x00\x00"
        chunk[offset + 4:offset + 8] = struct.pack("<I", size)
        chunk[offset + 8:offset + 16] = struct.pack("<Q", record_id)
        filetime = FILETIME_EPOCH_2026_11_01T09Z + (index * 30) * 10_000_000
        chunk[offset + 16:offset + 24] = struct.pack("<Q", filetime)
        chunk[offset + 24:offset + 24 + len(binxml)] = binxml
        chunk[offset + 24 + len(binxml):offset + size] = struct.pack("<I", size)
        offset += size

    string_table_offset = offset
    blob = names.blob
    chunk[offset:offset + len(blob)] = blob
    offset += len(blob)
    free_space = offset
    total = len(encoded)

    chunk[8:16] = struct.pack("<Q", 1)  # primeiro número de registro do chunk
    chunk[16:24] = struct.pack("<Q", total)  # último número de registro
    chunk[24:32] = struct.pack("<Q", 1)  # primeiro id
    chunk[32:40] = struct.pack("<Q", total)  # último id
    chunk[40:44] = struct.pack("<I", 128)  # header_size
    chunk[44:48] = struct.pack("<I", offsets[-1] if offsets else 0)
    chunk[48:52] = struct.pack("<I", free_space)

    # strings_offsets (primeiro slot) aponta para a primeira entrada da tabela.
    if blob:
        chunk[128:132] = struct.pack("<I", string_table_offset)
    # template_offsets fica zerado: os registros não usam template.

    events_checksum = zlib.crc32(bytes(chunk[CHUNK_HEADER_SIZE:free_space]))
    chunk[52:56] = struct.pack("<I", events_checksum & 0xFFFFFFFF)

    header_crc = zlib.crc32(bytes(chunk[0:120]) + bytes(chunk[128:512]))
    chunk[124:128] = struct.pack("<I", header_crc & 0xFFFFFFFF)

    return bytes(chunk), total


def build_file() -> bytes:
    chunk, total = build_chunk()
    header = bytearray(FILE_HEADER_SIZE)
    header[0:8] = b"ElfFile\x00"
    header[8:16] = struct.pack("<Q", 0)  # oldest chunk
    header[16:24] = struct.pack("<Q", 0)  # current chunk
    header[24:32] = struct.pack("<Q", total + 1)  # próximo record id
    header[32:36] = struct.pack("<I", 128)  # header_size
    header[36:38] = struct.pack("<H", 1)  # minor
    header[38:40] = struct.pack("<H", 3)  # major
    header[40:42] = struct.pack("<H", FILE_HEADER_SIZE)  # header_block_size
    header[42:44] = struct.pack("<H", 1)  # chunk_count
    header[124:128] = struct.pack("<I", zlib.crc32(bytes(header[0:120])) & 0xFFFFFFFF)
    return bytes(header) + chunk


def main() -> int:
    destination = (
        Path(sys.argv[1])
        if len(sys.argv) > 1
        else Path(__file__).resolve().parent.parent / "tests/fixtures/senai-demo.evtx"
    )
    data = build_file()
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)
    print(f"{destination}: {len(data)} bytes, {len(EVENTS)} eventos")
    print(f"sha256: {__import__('hashlib').sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
