# Auditoría inicial de la baseline

**Fecha:** 2026-08-13 (America/Panama)  
**Alcance:** paquete local `xunlie` antes de repositorio remoto y código de producto  
**Candidato:** working tree  
**Auditoría:** diseño/consistencia; no certificación externa

## Dictamen

El sistema de calidad es **estructuralmente consistente como propuesta**. G0 permanece **BLOCKED** y no existe aún una evaluación de calidad del producto, porque el producto no ha empezado a construirse.

## Evidencia ejecutada

| Prueba | Resultado |
|---|---|
| `python scripts/validate_quality_system.py` | PASS |
| requisitos con enlaces completos | 24/24 |
| invariantes/componentes | 12/8, todos enlazados y grafo acíclico |
| controles/gates | 28/8, referencias válidas |
| riesgos | 12/12 con requisitos, controles y tests |
| herramientas | 14/14 con owner, fuente y política de pin |
| parse de JSON/YAML | PASS |
| sintaxis Python | PASS |
| enlaces Markdown locales | PASS |
| workflow con `actionlint` | no ejecutado; herramienta no instalada |

El workflow fue parseado como YAML y es simple, pero la ausencia de `actionlint` se conserva como limitación, no como PASS equivalente.

## Evaluación G0

`quality/assessments/G0-initial.json` calcula completitud `80.0/100` y decisión `BLOCKED`.

- PASS: documentos/contratos, trazabilidad y arquitectura.
- BLOCKED: asignación nominal de roles, controles GitHub activos y decisiones de owner/visibilidad/licencia.
- Aprobador: ausente, correctamente, porque una baseline bloqueada no se aprueba.

El registro inicial no debe editarse a posteriori; una vez resueltos los blockers se crea un GateRecord nuevo.

## Estado de actualización

El pin del único Action activo se comprobó contra el repositorio oficial: `actions/checkout v7.0.1`, commit `3d3c42e5aac5ba805825da76410c181273ba90b1`, publicado el 2026-07-20. Dependabot deberá mantener el SHA.

La baseline normativa usa ediciones finales vigentes: IEEE 730-2026, ISO/IEC/IEEE 12207:2026, ISO/IEC 25010:2023, ISO/IEC 25040:2024, NIST SSDF 1.1 y SLSA 1.2. Un draft posterior se registra en el radar, pero no sustituye automáticamente la referencia final.

## Limitaciones y acciones

1. No existe remoto, ruleset ni prueba de rechazo de bypass: completar DEC-001/002 y `CTRL-REPO-001`.
2. No hay licencia: completar DEC-003 antes de publicar/copiar código externo.
3. Roles no tienen personas: completar DEC-004 y comprobar independencia real.
4. No hay código Rust: cobertura, mutación, SAST, build y release son controles planificados, no ejecutados.
5. Threat model es borrador: plataformas, adapter, telemetría y retención deben cerrarse en G2.
6. Ejecutar `actionlint` y `zizmor` al activar la lane de seguridad.

## Condición para nueva auditoría

Repetir G0 después de crear el remoto y resolver DEC-001 a DEC-004. El resultado esperado es `100/100`, todos los controles `PASS`, evidencia remota por digest y aprobación independiente.

