# ADR-0003: variantes de historia certificadas por recompilación

- Estado: propuesto para aprobación G3
- Fecha: 2026-08-14
- Decisor técnico: Chief Architect
- Requisitos: `REQ-F-004`, `REQ-Q-001`, `REQ-Q-002`
- Riesgos: `RISK-001`, `RISK-002`

## Contexto

Xunlie debe variar la presentación o el orden de una historia sin confundir una mutación
semántica con una variante legítima. Confiar en que el propio operador declare equivalencia
permitiría certificados circulares: el componente que hace el cambio también decidiría si el
cambio fue seguro.

M2 necesita además un artefacto portable que M3 pueda consumir sin ejecutar código del operador
ni depender del sistema de archivos que produjo la variante.

## Decisión

El engine expone una interfaz pura `VariantOperator` con tres responsabilidades limitadas:

1. declarar una identidad y versión estable;
2. evaluar y explicar todas sus precondiciones;
3. transformar una lista de fuentes en memoria.

El operador no emite certificados. `generate_certified_variant` compila primero la historia
original, ejecuta las precondiciones, aplica el operador, compila la historia resultante y compara
los `contentDigest`. Solo una igualdad exacta produce `EquivalenceCertificate`.

El certificado `xunlie.equivalence-certificate/v1` enlaza:

- operador y versión;
- resultado y explicación de cada precondición ejecutada;
- digest de los bytes e identidades de la historia antes y después;
- `contentDigest` y `artifactDigest` de ambos `ContractIR`;
- método de prueba `xunlie.recompile-and-compare/v1`;
- `contentDigest` canónico del propio certificado (expuesto por la CLI como
  `certificateDigest`).

La historia transformada y el certificado se persisten juntos como
`xunlie.certified-variant/v1`. El bundle tiene su propio productor y `contentDigest`, por lo que
una alteración de las fuentes se detecta antes del replay. El verificador no confía únicamente en
los campos persistidos: valida el contenedor y el certificado, recompila su historia, vuelve a
ejecutar el operador desde la historia baseline y exige igualdad byte a byte con todo el artefacto
regenerado.

## Operadores iniciales

- `json.presentation.normalize@1.0.0`: normaliza la representación JSON de todas las fuentes.
- `history.independent-adds.reverse@1.0.0`: invierte operaciones únicamente cuando existe una
  fuente, hay al menos dos operaciones, todas son `add` y sus requisitos son distintos.

Reemplazos, revocaciones, ids repetidos, inputs ya normalizados y cualquier transformación sin
cambio observable se excluyen con precondiciones fallidas y no producen artefacto parcial.

## Consecuencias

- Una implementación defectuosa o maliciosa de `VariantOperator` no puede autocertificar un
  cambio semántico.
- El certificado demuestra equivalencia respecto de `ContractIR v1`; no demuestra equivalencia
  de programas ni suficiencia de los oráculos de M4/M5.
- La verificación requiere la historia baseline exacta y la misma versión del operador.
- Cambiar la semántica de un operador exige otra versión; cambiar campos o significado del
  certificado exige política de compatibilidad o versión mayor del esquema.
- El digest del certificado aporta integridad y reproducibilidad, no identidad ni firma de un
  tercero. Las atestaciones de ejecución y release pertenecen a milestones posteriores.

## Alternativas descartadas

- Comparar solo texto: detecta diferencias de presentación pero no equivalencia contractual.
- Confiar en una bandera `equivalent` emitida por el operador: viola separación de autoridad.
- Guardar fuente y certificado en archivos independientes: permite pares inconsistentes y
  dificulta un hand-off verificable a M3.
- Permitir permutaciones arbitrarias: replacements y revocations pueden depender del orden.
