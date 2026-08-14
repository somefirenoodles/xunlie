# Registro de decisiones abiertas

Estas decisiones requieren autoridad del propietario. Las recomendaciones permiten avanzar sin ocultar supuestos.

| ID | Decisión | Recomendación | Bloquea | Estado |
|---|---|---|---|---|
| DEC-001 | Owner GitHub | `somefirenoodles` | — | aceptada |
| DEC-002 | Visibilidad | público | — | aceptada |
| DEC-003 | Licencia | MIT | — | aceptada |
| DEC-004 | Operación | OpenAI Codex ejecuta todos los roles; `somefirenoodles` conserva la titularidad del repositorio | — | aceptada con limitación |
| DEC-005 | Plataformas v1 | Linux x86_64 primario; macOS arm64 y Windows x86_64 para CLI/replay | G1/G2 | abierta |
| DEC-006 | Primer adapter real | protocolo genérico + adapter del agente usado por el equipo piloto | G2 | abierta |
| DEC-007 | Política de telemetría | local/off por defecto; export OpenTelemetry opt-in y redactado | G2 | abierta |
| DEC-008 | Retención/coste | evidencia de release por vida +24 meses; corridas PR 90 días salvo regulación | G2 | abierta |
| DEC-009 | Compatibilidad pública | garantizar lectura de bundle `N-1` desde la primera beta | G2 | abierta |
| DEC-010 | Distribución | binarios GitHub Releases + imagen OCI; package managers después de GA | G2/G5 | abierta |
| DEC-011 | Aprobación independiente | incorporar un segundo maintainer humano o auditor externo antes de afirmar independencia | G0/G5 | abierta |

## Preguntas de aprobación

1. ¿Qué agente y qué plataformas constituyen el piloto real?
2. ¿Se incorporará un segundo maintainer humano o un auditor externo antes de la primera release soportada?

## Limitación del modelo operativo

Codex puede diseñar, implementar, verificar, documentar y operar la automatización, pero no es una persona jurídica ni una identidad independiente de sí misma. Por tanto, la ejecución de todos los roles por Codex no satisface la separación autor–aprobador. La consistencia automática sigue siendo auditable; la aprobación independiente permanece bloqueada mediante DEC-011.

Resolver una fila exige ADR o registro firmado; no basta eliminarla de esta tabla.
