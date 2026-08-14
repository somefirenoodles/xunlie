# ADR-0002: evidencia append-only direccionada por contenido

- Estado: propuesto
- Fecha: 2026-08-13
- Decisores: Chief Architect, Quality Lead, Security Lead

## Contexto

El veredicto de Xunlie debe reconstruirse después de una release y resistir cambios de herramientas, índices y reportes.

## Decisión

JSON y JSONL direccionados por SHA-256 serán el registro primario. Un manifiesto enlaza inputs, outputs, eventos y atestaciones. SQLite puede usarse como índice local descartable y reconstruible, nunca como única evidencia.

## Consecuencias

- Los bundles son portables, comparables y aptos para firma/atestación.
- Las correcciones son nuevos eventos; no se modifica historia.
- Se requiere política explícita para secretos, redacción y retención.

