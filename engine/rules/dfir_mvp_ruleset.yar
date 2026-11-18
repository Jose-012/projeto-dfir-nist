// dfir-mvp-ruleset — ruleset YARA-X curado e versionado do MVP (F1-02).
//
// Todas as regras abaixo foram escritas originalmente para este projeto —
// nenhuma foi copiada de um ruleset de terceiros (YARA-Forge ou outro). Isso
// satisfaz a DECISÃO da seção 9.1 da spec canônica ("ruleset próprio e
// curado") e evita qualquer questão de proveniência/licença de regras
// importadas (seção 9.1, nota sobre YARA-Forge/GPL-3.0).
//
// Processo de curadoria seguido (seção 9.2): cada regra abaixo foi revisada
// manualmente, tem motivação registrada neste comentário e teste positivo e
// negativo correspondente em engine/src/yara_scan.rs (mod tests). Nenhuma
// atualização automática de regras é realizada — uma nova versão do ruleset
// só é criada por revisão humana explícita (ver RULESET_VERSION em
// engine/src/yara_scan.rs).
//
// Licença: Apache-2.0 (mesma licença do crate dfir-engine).
//
// Metadados obrigatórios por regra: description, confidence (LOW/MEDIUM/
// HIGH/CONFIRMED — mesmo vocabulário de docs/PROJECT_SPEC_v6_4_5.md seções
// 13-17 e do modelo analítico F1-01), severity, rule_version.
//
// Lembrete normativo (seção 9): nenhum match aqui é prova definitiva de
// atividade maliciosa — cada resultado é um indício estruturado, com
// evidência, regra, versão e confiança, para revisão humana.

rule eicar_test_string
{
    meta:
        description = "Detecta a string de teste padrao EICAR para antivirus (arquivo de teste inofensivo, nao e malware real)"
        confidence = "CONFIRMED"
        severity = "INFO"
        reference = "https://www.eicar.org/download-anti-malware-testfile/"
        rule_version = "1"
    strings:
        $eicar = "X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*"
    condition:
        $eicar
}

rule suspicious_powershell_encoded_command
{
    meta:
        description = "PowerShell invocado com flag de comando codificado junto de um bloco longo em formato base64; padrao comum de ofuscacao, nao e prova de intencao maliciosa"
        confidence = "MEDIUM"
        severity = "MEDIUM"
        rule_version = "2"
    strings:
        // `wide ascii`: texto de eventos do Windows (EVTX/BinXML) é UTF-16LE;
        // sem `wide` a regra nunca casava em .evtx (AUD-09).
        $flag1 = "-EncodedCommand" nocase wide ascii
        $flag2 = "-enc " nocase wide ascii
        $flag3 = "-e " nocase wide ascii
        $b64_run = /[A-Za-z0-9+\/]{200,}={0,2}/
        $b64_run_wide = /[A-Za-z0-9+\/]{200,}={0,2}/ wide
    condition:
        ($flag1 or $flag2 or $flag3) and ($b64_run or $b64_run_wide)
}

rule generic_ransom_note_keywords
{
    meta:
        description = "Combinacao de frases comumente vistas em notas de resgate de ransomware; heuristica generica, nao e prova de incidente ativo"
        confidence = "LOW"
        severity = "HIGH"
        rule_version = "2"
    strings:
        // Notas de resgate em UTF-16LE (Bloco de Notas "Unicode") também casam.
        $files = "your files" nocase wide ascii
        $encrypted = "encrypted" nocase wide ascii
        $decrypt = "decrypt" nocase wide ascii
        $bitcoin = "bitcoin" nocase wide ascii
        $wallet = "wallet" nocase wide ascii
    condition:
        $files and $encrypted and $decrypt and ($bitcoin or $wallet)
}

rule mimikatz_command_reference
{
    meta:
        description = "Referencia a um comando conhecido de dump de credenciais associado ao Mimikatz (ex.: encontrado em logs, linhas de comando ou strings de memoria)"
        confidence = "HIGH"
        severity = "HIGH"
        reference = "https://attack.mitre.org/software/S0002/"
        rule_version = "2"
    strings:
        $cmd1 = "sekurlsa::logonpasswords" nocase wide ascii
        $cmd2 = "sekurlsa::pth" nocase wide ascii
        $cmd3 = "lsadump::sam" nocase wide ascii
    condition:
        any of them
}
