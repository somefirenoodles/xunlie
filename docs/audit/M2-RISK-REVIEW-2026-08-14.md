# Revisión dirigida M2 — RISK-001 y RISK-002

**Fecha:** 2026-08-14
**Alcance:** corte vertical de variantes certificadas
**Decisión técnica:** apto para revisión G3; la aprobación independiente no se suplanta con este documento

## RISK-001 — Falsa equivalencia contractual

**Control implementado:** el operador propone, pero el engine compila ambas historias y compara el
`contentDigest`; el certificado no se emite cuando difieren. La verificación vuelve a ejecutar el
operador y compara el artefacto completo byte a byte.

**Evidencia reproducible:**

- pruebas unitarias de invariantes y tampering en `xunlie-domain::variant`;
- `operator_cannot_self_certify_a_semantic_mutation` prueba separación de autoridad;
- `independent_add_reversal_is_metamorphically_equivalent` implementa `TEST-VARIANT-META`;
- `certified_variant_v1_matches_golden_vector` fija serialización y digests;
- E2E CLI cubre certificación, replay, exclusión y alteración.

Campaña local final:

```text
cargo mutants --in-place --no-shuffle --minimum-test-timeout 20
90 mutants evaluated: 61 caught, 29 unviable, 0 missed
```

La selección se fija en `.cargo/mutants.toml` y el mismo comando se ejecuta en
`mutation-certified-variants` dentro de `deep-quality.yml`.

**Riesgo residual:** un defecto compartido por el compilador before/after podría conservar el mismo
digest incorrecto. M2 reduce el riesgo con propiedades, golden y mutación, pero requiere revisión
independiente G3 y ampliación continua del corpus.

## RISK-002 — Confianza falsa por oráculo insuficiente

**Resultado:** no se declara mitigado por M2. La igualdad de `ContractIR` comprueba equivalencia de
la especificación efectiva, no suficiencia de tests ni equivalencia de implementaciones producidas
por agentes. El formato lo hace explícito mediante `xunlie.recompile-and-compare/v1` y no contiene
un veredicto de ejecución.

**Control temporal:** ninguna API de M2 emite PASS de agente o PDR. M4 debe registrar cobertura por
obligación y M5 debe producir `inconclusive` cuando sea insuficiente.

## Condiciones para GO de G3

1. `cargo xtask quality` en verde en plataforma soportada;
2. campaña `cargo-mutants` dirigida sin mutantes omitidos en la lógica crítica acordada;
3. reproducción del golden y los E2E por un revisor distinto del autor;
4. sin findings críticos/altos abiertos sobre `INV-ARCH-002` o `REQ-F-004`.
