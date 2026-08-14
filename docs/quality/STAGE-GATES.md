# Gates y criterios de salida

Este documento define la revisión previa a finalizar cada etapa. Los identificadores y controles mínimos se validan contra `quality/stages.json`.

## Protocolo común

1. El dueño de etapa congela el conjunto candidato y genera evidencia.
2. El validador calcula aplicabilidad y comprueba integridad.
3. Quality muestrea evidencia, reproduce controles críticos y registra findings.
4. El dueño corrige o solicita un waiver válido.
5. Un aprobador independiente emite `GO`, `REWORK` o `BLOCKED`.
6. Solo `GO` permite declarar la etapa terminada; para PR se usa `MERGE`.

La evidencia no puede crearse después de la aprobación para justificarla retroactivamente.

## G0 — Gobierno y baseline

**Entrada:** propósito y sponsor identificados.  
**Obligatorio:** SQ Plan, arquitectura, ADR iniciales, requisitos, RACI, taxonomía de riesgos, esquema de trazabilidad, repositorio/rulesets y validador en verde.  
**Muestra de auditoría:** reconstruir tres requisitos elegidos al azar y comprobar independencia de aprobación.  
**Salida:** baseline firmada/versionada y backlog de construcción autorizado.

## G1 — Descubrimiento y requisitos

**Entrada:** problema/hipótesis con usuario y resultado.  
**Obligatorio:** requisitos atómicos, criterios observables, prioridades, supuestos, out-of-scope, riesgos, datos/oráculos y matriz inicial.  
**Controles:** ambigüedad, contradicción, duplicidad, verificabilidad, privacidad y abuso.  
**Salida:** baseline de requisitos apta para arquitectura; incertidumbres pasan a spikes con fecha.

## G2 — Arquitectura y riesgo

**Entrada:** requisitos aprobados.  
**Obligatorio:** vistas, fronteras de confianza, responsabilidades, ADR, threat model, contratos, compatibilidad, fitness functions, estrategia de pruebas y estimación de presupuesto.  
**Controles:** cada requisito tiene componente; cada componente tiene responsabilidad y dueño; cada riesgo tiene control y prueba.  
**Salida:** arquitectura implementable sin decisiones críticas implícitas.

## G3 — Construcción por incremento/PR

**Entrada:** Definition of Ready satisfecha.  
**Obligatorio:** diff enfocado, trazabilidad actualizada, tests, documentación, análisis de arquitectura, dependencias, SAST, secretos y revisión CODEOWNER.  
**Controles específicos:** formato, lint, unit/property/contract tests, cobertura diferencial, compatibilidad, licencias y workflow security.  
**Salida:** commit integrado en `main`, evidencia ligada a SHA y sin deuda anónima.

Para cambios de dominio, protocolo, seguridad o CI se exige segundo aprobador y mutation/property tests aplicables.

## G4 — Integración y validación

**Entrada:** alcance de versión integrado.  
**Obligatorio:** 100% de requisitos cubiertos, E2E, corpus dorado, matrices de OS, rendimiento, caos, replay, threat model verificado y manuales.  
**Controles:** ninguna regresión Sev-1/2, flakiness requerida cero, PDR de baselines explicado y evidencia reproducible.  
**Salida:** release candidate inmutable.

## G5 — Release candidate

**Entrada:** digest exacto del RC.  
**Obligatorio:** SBOM, checksums, provenance, firmas/atestaciones, notas, migración/rollback, soporte y aprobación de seguridad/calidad.  
**Controles:** instalación limpia, verificación independiente de artefacto, licencia, vulnerabilidades, reproducibilidad y smoke tests.  
**Salida:** promoción del mismo digest y registro de release.

## G6 — Operación y mantenimiento

**Entrada:** release desplegada/publicada.  
**Cadencia:** continua para alertas, mensual para dependencias y trimestral para arquitectura/estado del arte.  
**Obligatorio:** SLO, incidentes, vulnerabilidades, compatibilidad, tendencias, deuda, adopción y eficacia de CAPA.  
**Salida:** continuar, mitigar con fecha o rollback/deprecación.

## G7 — Retiro

**Entrada:** decisión de deprecación.  
**Obligatorio:** inventario de usuarios/versiones, migración, comunicación, exportación, borrado/retención, revocación de secretos y archivo verificable.  
**Salida:** producto retirado sin dependencia huérfana ni evidencia perdida.

## Causas automáticas de bloqueo

- requisito aplicable sin prueba o componente;
- invariante arquitectónico violado;
- evidencia ausente, alterada o no ligada al commit;
- vulnerabilidad crítica/alta abierta en release;
- secreto confirmado;
- aprobación propia o bypass no justificado;
- waiver vencido;
- firma, checksum o procedencia inválidos;
- resultado crítico flaky o no reproducible;
- decisión abierta clasificada como bloqueante.

