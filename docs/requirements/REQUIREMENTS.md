# Baseline inicial de requisitos

**ID:** `XUNLIE-REQ-001`  
**Versión:** `0.1.0`  
**Estado:** propuesta

`quality/requirements.json` es la representación ejecutable. Este documento resume intención y aceptación; toda modificación semántica debe actualizar ambos en la misma PR.

## Requisitos funcionales

| ID | Capacidad | Aceptación resumida |
|---|---|---|
| REQ-F-001 | Ingesta de fuentes | acepta fuentes soportadas, conserva procedencia y rechaza input inválido con diagnóstico |
| REQ-F-002 | Compilación ContractIR | produce IR versionado, validado y con digest canónico |
| REQ-F-003 | Resolución de historias | aplica precedencia explícita y reporta conflictos sin adivinar intención |
| REQ-F-004 | Variantes certificadas | genera solo historias cuyas precondiciones y equivalencia sean verificables |
| REQ-F-005 | Ejecución aislada | crea workspace limpio por corrida y evita contaminación cruzada |
| REQ-F-006 | Protocolo de adaptadores | negocia versión/capacidades y clasifica incompatibilidad antes de ejecutar |
| REQ-F-007 | Verificadores enchufables | ejecuta oráculos con alcance, versión y observaciones tipadas |
| REQ-F-008 | Evidencia | persiste manifiesto, eventos, outputs y digests de forma append-only |
| REQ-F-009 | Comparación y PDR | separa divergencia semántica, fallo de oráculo e infraestructura |
| REQ-F-010 | Reporte y gate | entrega veredicto explicable, granular y con códigos de salida estables |
| REQ-F-011 | Presupuesto/autoridad | aplica límites y permisos externamente al agente |
| REQ-F-012 | Replay | reconstruye un veredicto desde bundle compatible sin invocar nuevamente al agente |

## Requisitos de calidad

| ID | Atributo | Aceptación resumida |
|---|---|---|
| REQ-Q-001 | Determinismo | misma entrada/versiones producen mismo contrato, variantes y decisión |
| REQ-Q-002 | Auditabilidad | una muestra de release se traza de requisito a artefacto y aprobación |
| REQ-Q-003 | Seguridad/aislamiento | permisos mínimos, sin contaminación ni exfiltración en pruebas adversariales |
| REQ-Q-004 | Confiabilidad | fallos parciales se clasifican y recuperan sin producir falsos PASS |
| REQ-Q-005 | Eficiencia | límites de tiempo/memoria/coste se miden y aplican |
| REQ-Q-006 | Portabilidad | CLI y replay pasan en plataformas soportadas |
| REQ-Q-007 | Compatibilidad | formatos/protocolos compatibles sobreviven upgrades; rupturas fallan temprano |
| REQ-Q-008 | Mantenibilidad | capas, complejidad, pruebas y dependencias cumplen fitness functions |
| REQ-Q-009 | Flexibilidad | un adaptador conforme se integra sin modificar el dominio |
| REQ-Q-010 | Interacción | diagnósticos indican causa, localización, reparación y código de salida |
| REQ-Q-011 | Privacidad | secretos y contenido sensible se minimizan/redactan y respetan retención |
| REQ-Q-012 | Integridad supply chain | release incluye SBOM, checksum y procedencia verificable |

## Política de cambios

- Un requisito es atómico, necesario, factible, no ambiguo y verificable.
- `proposed → approved → implemented → verified → retired` es el flujo permitido.
- Cambiar aceptación después de implementar requiere análisis de impacto; no se edita para hacer pasar un resultado existente.
- Ningún requisito se marca `verified` sin prueba y evidencia ligadas a una versión.

