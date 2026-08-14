# Threat model inicial

**Estado:** borrador para G2.  
**Método:** fronteras de confianza + STRIDE + casos de abuso agéntico.  
**Referencia:** OWASP Top 10 for Agentic Applications 2026 y NIST SSDF.

## Activos

- integridad de ContractIR, certificados y reglas de gate;
- repositorio/workspace del proyecto evaluado;
- credenciales, código privado y datos en prompts/logs;
- evidencia, veredictos y artefactos de release;
- presupuesto de compute, red, tokens y dinero;
- identidad/versiones de agente, adapter y verificador.

## Fronteras

1. usuario/CI → CLI;
2. fuentes no confiables → compiler/domain;
3. core → adapter fuera de proceso;
4. runner → sandbox/workspace;
5. workspace no confiable → verificadores;
6. eventos/outputs → evidence store;
7. evidence/report → GitHub gate/release.

## Amenazas y controles

| Amenaza | Impacto | Controles | Prueba |
|---|---|---|---|
| Instrucción maliciosa obtiene autoridad | ejecución/exfiltración | separar contexto/autoridad, allowlist, mínimos permisos | TEST-ISOLATION-ADV |
| Path traversal/symlink escapa workspace | host comprometido | paths canónicos, filesystem sandbox, adversarial fixtures | TEST-ISOLATION-ADV |
| Adapter suplanta versión/capacidad | veredicto inválido | handshake, digest, proceso aislado, conformance | TEST-PROTOCOL-CONFORMANCE |
| Agente evade presupuesto o deja procesos | coste/contaminación | supervisor externo, cgroups/job objects, cleanup | TEST-BUDGET-CHAOS |
| Output manipula logs/reporte | ocultamiento/inyección | framing JSONL, escaping, límites, separación stdout/logs | TEST-CLI-DIAGNOSTIC |
| Evidencia se reescribe o omite | auditoría falsa | append-only, manifiesto, hash, atestación | TEST-EVIDENCE-INTEGRITY |
| Verificador débil concede PASS | confianza falsa | alcance de oráculo, cobertura, inconclusive, diversidad | TEST-VERIFIER-CONTRACT |
| Dependencia/action comprometida | ejecución en CI/release | pin SHA, allowlist, cargo-deny, SBOM/provenance | TEST-RELEASE-VERIFY |
| Prompt/log conserva secreto | divulgación | minimización, redacción, retención, no telemetry default | TEST-REDACTION |
| Drift de modelo cambia baseline | comparación inválida | identidad/versiones, repetición, distribución, replay | TEST-REPLAY-CROSS |
| Fallo parcial se vuelve PASS | release incorrecta | estados tipados y fail closed | TEST-CHAOS-FAILCLOSED |
| Maintainer elude controles | supply chain/gobierno | rulesets, dos personas, logs y break-glass | TEST-AUDIT-RECONSTRUCT |

## Supuestos por validar

- el backend de sandbox soportará primitivas equivalentes en plataformas objetivo;
- el agente no necesita credenciales de mayor alcance que el proyecto piloto;
- outputs completos pueden contener datos sensibles y se tratan como no confiables;
- el evidence store local necesita backup/retención definidos por despliegue.

## Criterio G2

Cerrar DEC-005 a DEC-008, asignar dueño nominal, probar cada amenaza crítica y aceptar riesgo residual. Este borrador no satisface por sí solo `CTRL-SEC-004`.

