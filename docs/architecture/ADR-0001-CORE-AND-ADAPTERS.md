# ADR-0001: núcleo Rust y adaptadores fuera de proceso

- Estado: propuesto
- Fecha: 2026-08-13
- Decisores: Product Owner, Chief Architect, Quality Lead, Security Lead

## Contexto

Xunlie necesita resultados deterministas y auditables, pero debe integrar agentes, modelos, sandboxes y verificadores que cambian con rapidez y poseen dependencias incompatibles.

## Decisión

Se implementará el núcleo y la CLI en Rust 2024. Los adaptadores de agente y verificador se ejecutarán fuera de proceso y se comunicarán mediante un protocolo JSONL versionado. No se permiten plugins dinámicos dentro del proceso de confianza.

## Consecuencias

- Se reduce la superficie de memoria y distribución del núcleo.
- Un adaptador puede usar otro lenguaje sin contaminar dependencias del producto.
- La compatibilidad debe probarse mediante negociación y suite de conformidad.
- Hay coste de serialización y operación de procesos, aceptado a cambio de aislamiento y reemplazabilidad.

## Alternativas descartadas

- TypeScript end-to-end: integración rápida, pero mayor superficie de runtime y menos control del núcleo determinista.
- Plugins nativos dinámicos: menor overhead, pero ABI frágil y aislamiento insuficiente.
- Microservicios desde v1: añaden operación distribuida sin necesidad de dominio.

