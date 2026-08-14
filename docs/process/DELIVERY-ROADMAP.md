# Roadmap de construcción gobernada

Las duraciones son rangos de planificación, no compromisos. Cada milestone termina con una demostración y un gate; el siguiente puede explorar en paralelo, pero no basarse en una salida no aprobada.

## M0 — Activar gobierno (1 semana)

- resolver DEC-001 a DEC-004;
- crear repositorio, aplicar rulesets y asignar CODEOWNERS;
- aprobar SQ Plan, arquitectura, requisitos y ADR;
- ejecutar pruebas adversariales del ruleset;
- emitir nuevo GateRecord G0 con `100/100` y decisión `GO`.

## M1 — Contrato y resolución (2–3 semanas)

- workspace Rust, `domain`, `engine`, `cli`, `testkit`;
- ContractIR v1, canonicalización y diagnósticos;
- álgebra de precedencia/conflicto;
- golden corpus, property tests y fuzz de parser;
- activar fmt, Clippy, tests, coverage, cargo-deny y CodeQL.

Gate: G1 de requisitos y G2 para esta slice; cada PR pasa G3.

## M2 — Variantes certificadas (2 semanas)

- interfaz `VariantOperator` y primeros operadores;
- precondiciones ejecutables y `EquivalenceCertificate`;
- pruebas metamórficas/mutación;
- explicación de variantes excluidas.

Gate: G3 + revisión dirigida de RISK-001/RISK-002.

Estado del corte vertical: implementado. Incluye dos operadores deterministas, certificado v1,
replay independiente, exclusiones explicadas, vector golden, propiedades metamórficas y campaña de
mutación dirigida. La aprobación formal G3 continúa requiriendo un revisor independiente.

## M3 — Protocolo y ejecución aislada (3–4 semanas)

- protocolo JSONL, handshake y conformance kit;
- backend Git worktree + contenedor OCI inicial;
- scheduler, límites y cancelación;
- agente falso determinista y primer adapter real;
- threat model y pruebas adversariales/caos.

Gate: G2 actualizado + G3; no integrar adapter real antes de aislar permisos.

## M4 — Verificadores y evidencia (2–3 semanas)

- verificadores de tests, estático, estructura y restricciones;
- EvidenceBundle, digests, redacción y replay;
- clasificación tipada de fallos;
- auditoría de una corrida completa.

Gate: G3 y preevaluación G4.

## M5 — Comparación, reporte y CI gate (2 semanas)

- PDR, distribuciones, cobertura de oráculo y atribución;
- reportes JSON/terminal y códigos de salida;
- GitHub Action/CLI CI;
- casos canónico fallido e inconclusive.

Gate: G4 sobre corpus piloto.

## M6 — Beta endurecida (2–4 semanas)

- matrices de plataforma, rendimiento, caos, fuzz/mutation extensos;
- documentación operativa, soporte y rollback;
- SBOM, procedencia, firma/atestación y verificación independiente;
- auditoría de release y piloto con proyectos reales.

Gate: G5. La primera publicación soportada ocurre solo aquí.

## Criterios de priorización

Primero se construyen los mecanismos que pueden demostrar falsedad: resolvedor, certificados, aislamiento, oráculos y evidencia. Dashboard, integraciones adicionales y optimización llegan después de que el veredicto sea confiable.

## Estimación de capacidad

Equipo mínimo razonable: 2 desarrolladores Rust/sistemas, 1 Quality/SDET con propiedades/fuzzing, participación de Architect/Security y Product. En un equipo menor, mantener al menos dos personas para independencia; si una sola persona construye, un revisor externo debe aprobar gates críticos.
