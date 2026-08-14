# Modelo de auditoría

## Objetivo

Una auditoría de Xunlie no pregunta solo “¿existe el documento?”, sino “¿el control operó sobre el artefacto exacto y su evidencia permite reproducir la decisión?”.

## Tipos

| Auditoría | Cadencia | Alcance | Independencia |
|---|---|---|---|
| Automática | cada PR/gate | estructura, reglas, tests, seguridad, evidencia | herramienta + revisión del resultado |
| Muestreo interno | mensual | PR, waivers, findings, trazabilidad y métricas | Quality no autor de la muestra |
| Readiness | G4/G5 | release completa, rollback y supply chain | segundo revisor/release verifier |
| Arquitectura | trimestral | dependencias, responsabilidades, deuda y ADR | Architect + Quality |
| Externa | anual o contractual | sistema de gestión y muestra de releases | auditor sin responsabilidad operativa |

## Método de muestreo

Cada auditoría combina:

- 100% de cambios críticos, waivers, Sev-1/2 y break-glass;
- muestra aleatoria reproducible de al menos 10% de PR del periodo o 5 PR, lo que sea mayor;
- muestra dirigida por señales: cambios grandes, alta churn, dependencia nueva, agente nuevo o score cercano a umbral;
- un recorrido inverso desde artefacto de release hasta necesidad;
- un recorrido directo desde requisito seleccionado hasta evidencia en release.

La semilla, población y consulta de selección se registran para impedir cherry-picking.

## Pruebas de auditoría

1. **Existencia:** el artefacto requerido está presente.
2. **Integridad:** digest/atestación corresponde al contenido.
3. **Pertinencia:** la evidencia observa la obligación correcta.
4. **Temporalidad:** se produjo antes de la aprobación.
5. **Independencia:** autor, cada revisor y verifier son instancias separadas; el registro demuestra
   tarea, permisos, candidato y veredicto, y el orquestador no cuenta como voto.
6. **Reproducibilidad:** control crítico se reejecuta con mismo resultado o variación explicada.
7. **Cierre:** finding/CAPA posee prueba de eficacia.

## Finding

Un finding registra ID, control, evidencia, severidad, condición observada, criterio violado, causa preliminar, dueño y fecha. Se evita una recomendación genérica: debe existir una condición verificable de cierre.

Clasificación:

- `Major`: invalida release/gate o revela fallo sistémico;
- `Minor`: incumplimiento acotado sin resultado falso;
- `Observation`: señal preventiva, no incumplimiento;
- `Opportunity`: mejora sin obligación normativa.

Un `Major` bloquea. Tres `Minor` con causa común abren CAPA sistémica.

## Paquete mínimo por release

- GateRecords G1-G5 aplicables;
- baseline de requisitos/arquitectura y ADR;
- commit/tag y diff de alcance;
- resultados de test/cobertura/mutación/replay;
- findings, waivers y CAPA;
- threat model y seguridad;
- lockfiles, dependency policy y SBOM;
- build provenance, checksums, firmas/atestaciones;
- instalación, smoke, rollback y aprobación independiente;
- notas de release y matriz de soporte.

## Criterio “perfecto”

La auditoría del gate es perfecta (`100/100`) cuando la población aplicable está completa, cada control tiene evidencia válida, toda la trazabilidad es bidireccional, no hay blocker y la aprobación es independiente. No afirma ausencia absoluta de defectos; afirma que el proceso definido operó sin excepción oculta.
