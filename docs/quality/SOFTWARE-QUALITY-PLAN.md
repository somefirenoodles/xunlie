# Software Quality Plan de Xunlie

**ID:** `XUNLIE-SQP-001`  
**Versión:** `0.1.0`  
**Estado:** propuesta para aprobación  
**Vigencia:** desde la aprobación de Gate 0  
**Propietario:** Quality Lead  
**Aprobadores:** Product Owner, Chief Architect, Quality Lead, Security Lead  
**Revisión:** por release, ante cambio mayor de riesgo/arquitectura y al menos trimestral

## 1. Objetivo

Este plan convierte la calidad de Xunlie en un sistema continuo, verificable y auditable. Controla en paralelo la intención, la arquitectura, la implementación, la seguridad y la evidencia; además impone una decisión formal antes de cerrar cada etapa.

El resultado buscado es software:

- **auditable:** cada decisión y veredicto se reconstruye desde entradas versionadas y evidencia inmutable;
- **escalable:** responsabilidades y límites permanecen estables al crecer equipos, proveedores y volumen;
- **iterable:** cada incremento pequeño atraviesa el mismo ciclo de trazabilidad y verificación;
- **actualizable:** herramientas y técnicas nuevas entran mediante evaluación, pin, piloto y rollback;
- **confiable:** los gates críticos son binarios y no pueden compensarse con promedios.

## 2. Alcance

Aplica a código de producto, protocolos, esquemas, documentación normativa, infraestructura CI, dependencias, adaptadores, fixtures, datasets de evaluación, artefactos de release y procedimientos operativos.

Aplica desde una propuesta de requisito hasta retiro del producto. Incluye contribuciones humanas y producidas con agentes. La autoría agéntica no reduce revisión, trazabilidad ni responsabilidad humana.

Quedan fuera solo prototipos en ramas o repositorios explícitamente desechables que no alimenten una decisión, release o baseline. Si un prototipo se reutiliza, debe pasar retrospectivamente todos los controles antes de integrarse.

## 3. Base normativa y adaptación

El sistema se alinea con:

- [ISO/IEC/IEEE 12207:2026](https://www.iso.org/standard/90219.html), para procesos del ciclo de vida;
- [IEEE 730-2026](https://standards.ieee.org/ieee/730/10854/), para planificación y ejecución del aseguramiento de calidad;
- [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html), para el modelo de calidad de producto;
- [ISO/IEC 25040:2024](https://www.iso.org/standard/83467.html), para el proceso de evaluación;
- [NIST SSDF SP 800-218 v1.1](https://csrc.nist.gov/pubs/sp/800/218/final), para prácticas de desarrollo seguro;
- [SLSA v1.2](https://slsa.dev/spec/v1.2/), para integridad de fuente y procedencia de build;
- [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/download/52117/?tmstv=1765059207), como taxonomía de amenazas agénticas.

Este plan no declara certificación formal contra esos estándares. Adopta controles proporcionados al riesgo de Xunlie y registra desviaciones mediante ADR o waiver.

## 4. Fuentes de verdad y precedencia

Cuando dos artefactos discrepan, rige esta precedencia:

1. requisito y criterio de aceptación aprobados;
2. invariante de seguridad o arquitectura aprobado;
3. ADR aceptado más reciente;
4. contrato/esquema machine-readable;
5. implementación y tests;
6. documentación explicativa.

Una contradicción no se resuelve silenciosamente por precedencia: se abre un defecto `CONFLICT`, se bloquea el gate y se corrigen todos los artefactos afectados.

## 5. Modelo de control de calidad

### 5.1 Dos bucles obligatorios

**Bucle continuo por cambio:** cada issue y pull request ejecuta trazabilidad, fitness functions, análisis estático, tests, seguridad y generación de evidencia. Evita acumular deuda hasta el final de una etapa.

**Gate de salida de etapa:** uno o más revisores independientes del autor verifican criterios,
evidencia, riesgos y residuales sobre un candidato congelado. La independencia puede realizarse
mediante instancias de agente separadas conforme a `xunlie.review-orchestration/v1`; no se reduce a
que “CI esté verde”.

### 5.2 Regla no compensatoria

Cada control es `MANDATORY`, `RISK_BASED` o `INFORMATIONAL`.

- Un `MANDATORY` fallido o sin evidencia produce `BLOCKED`.
- Un `RISK_BASED` omitido exige riesgo aceptado, dueño y vencimiento.
- Un `INFORMATIONAL` no bloquea, pero su tendencia puede crear una acción correctiva.
- Solo después de pasar todos los controles aplicables se calcula el score de completitud.
- `100/100` equivale a todos los controles aplicables pasados, 100% de evidencia válida y cero waivers vencidos.

No existe “aprobado con 95” si falla una propiedad crítica.

### 5.3 Quality lanes paralelos

| Lane | Dueño | Salida continua | Gate asociado |
|---|---|---|---|
| Intención y requisitos | Product Owner | criterios verificables, riesgos, prioridad | G1/G3 |
| Arquitectura | Chief Architect | ADR, mapa de componentes, fitness functions | G2/G3 |
| Construcción | Maintainer | código pequeño, revisable y versionado | G3 |
| Verificación | Quality Lead | pruebas independientes y cobertura de requisitos | G3/G4 |
| Seguridad/supply chain | Security Lead | threat model, findings, SBOM, procedencia | G2-G6 |
| Evidencia/auditoría | Quality Lead | bundle íntegro y decisión reproducible | todos |

## 6. Organización y separación de funciones

Las responsabilidades detalladas están en `docs/process/ROLES-RACI.md`.

Reglas mínimas:

- el autor no aprueba su propio cambio;
- una instancia revisora no participa en la autoría ni modifica el candidato antes de su veredicto;
- cambios críticos reciben al menos dos revisiones separadas y un resultado positivo requiere
  unanimidad; cualquier finding crítico/alto produce `REWORK` o `BLOCKED`;
- el Quality Lead puede bloquear cualquier gate por evidencia insuficiente;
- arquitectura, dominio, seguridad, workflows y releases requieren CODEOWNER;
- una excepción de seguridad no puede aprobarla quien la solicita;
- el Release Manager no modifica artefactos después de su atestación;
- un equipo pequeño puede acumular roles, pero no puede eliminar la revisión independiente de un cambio crítico.

La independencia aquí es operacional y auditable: identidad de instancia, tarea, alcance, SHA,
comandos, findings, hora y veredicto quedan registrados. El orquestador agrega los veredictos, pero
no aporta un voto revisor. El propietario humano conserva las decisiones que impliquen intención,
responsabilidad jurídica o aceptación de riesgo de negocio.

## 7. Plan por ciclo de vida

Los criterios ejecutables residen en `quality/stages.json`; el detalle humano está en `STAGE-GATES.md`.

| Gate | Etapa | Pregunta de decisión | Resultado permitido |
|---|---|---|---|
| G0 | Gobierno | ¿Existe baseline coherente y responsables explícitos? | GO/BLOCKED |
| G1 | Descubrimiento | ¿Cada necesidad es verificable, trazable y priorizada? | GO/REWORK/BLOCKED |
| G2 | Arquitectura | ¿El diseño realiza requisitos y contiene riesgos? | GO/REWORK/BLOCKED |
| G3 | Incremento | ¿El cambio preserva contrato, límites y calidad? | MERGE/BLOCKED |
| G4 | Integración | ¿El sistema satisface requisitos en entornos soportados? | RC/BLOCKED |
| G5 | Release candidate | ¿El artefacto exacto es seguro, reproducible y operable? | RELEASE/BLOCKED |
| G6 | Operación | ¿La versión continúa dentro de SLO, riesgo y soporte? | CONTINUE/MITIGATE/ROLLBACK |
| G7 | Retiro | ¿Datos, usuarios y evidencia tienen transición controlada? | RETIRE/BLOCKED |

Una etapa puede ser iterativa y solaparse con otra, pero su salida no se declara mientras su gate esté abierto.

## 8. Gestión de requisitos y trazabilidad

### 8.1 Identidad

- funcional: `REQ-F-nnn`;
- calidad: `REQ-Q-nnn`;
- invariante: `INV-ARCH-nnn`;
- riesgo: `RISK-nnn`;
- ADR: `ADR-nnnn`;
- prueba/control: `TEST-*` / `CTRL-*`;
- evidencia: `EVID-*`;
- excepción: `WAIVER-*`.

### 8.2 Definition of Ready

Un trabajo entra a construcción solo si posee requisito, valor, criterio de aceptación observable, riesgos, componentes previstos, impacto de compatibilidad y estrategia de verificación. Incertidumbre real puede registrarse como spike time-boxed, con pregunta y criterio de salida.

### 8.3 Definition of Done

Un cambio está terminado solo cuando:

1. satisface criterios de aceptación;
2. actualiza la trazabilidad bidireccional requisito → componente → prueba → evidencia → gate;
3. preserva o modifica formalmente la arquitectura;
4. pasan controles de código, seguridad y supply chain;
5. documentación, esquemas y ejemplos están sincronizados;
6. no crea findings críticos/altos ni waivers vencidos;
7. tiene revisión independiente y evidencia enlazada al commit.

La cobertura de requisitos debe ser 100%. Cobertura de líneas no sustituye cobertura de intención.

## 9. Consistencia arquitectónica

Toda PR declara `architecture_impact: none|compatible|adr-required`. `none` se valida contra el diff; una declaración falsa bloquea el merge.

Controles obligatorios:

- grafo de dependencias y capas;
- prohibición de IO/tiempo/red/azar en dominio;
- `unsafe` prohibido en crates de confianza;
- compatibilidad de esquemas y protocolos;
- snapshots de canonicalización y replay;
- presupuesto de dependencias y tamaño del binario;
- threat model si cambia una frontera de confianza;
- ADR aceptado antes del código para decisiones irreversibles.

El Chief Architect revisa trimestralmente la deriva agregada, incluso si todas las PR individuales pasaron.

## 10. Verificación y validación

### 10.1 Pirámide específica

- unitarias para tipos, errores y reglas locales;
- propiedades para resolución, precedencia, idempotencia y operadores;
- metamórficas para equivalencia entre historias;
- contract tests para protocolo y adapters;
- integración para pipeline y evidence bundles;
- end-to-end con agentes falsos deterministas antes de agentes reales;
- replay de corpus dorado;
- fuzzing de parsers, esquemas y eventos;
- mutación en lógica crítica;
- rendimiento y soak para scheduling/evidencia;
- caos controlado para timeout, proceso caído, red y disco;
- aceptación sobre casos de usuario y amenazas agénticas.

### 10.2 Independencia y oráculos

El autor implementa tests del cambio; Quality define o revisa pruebas de aceptación y propiedades críticas. Los oráculos tienen alcance explícito. Un test que no observa una obligación no cuenta como cobertura de ese requisito.

### 10.3 Flakiness

No se reintenta silenciosamente un test para obtener verde. Un test flaky se etiqueta, investiga y puede ponerse en cuarentena solo con dueño, issue, impacto y caducidad máxima de 14 días. Ninguna prueba requerida para un requisito crítico puede permanecer en cuarentena al liberar.

## 11. Umbrales de calidad

Los umbrales iniciales están en `quality/quality-plan.json` y se revisan con datos, nunca se reducen para hacer pasar una release.

- trazabilidad de requisitos aplicables: 100%;
- controles obligatorios con evidencia: 100%;
- cobertura global de línea/rama: ≥90%/≥85%;
- módulos críticos de dominio: 100% de decisiones definidas y mutation score ≥95%;
- resto del producto: mutation score ≥85% en campañas programadas;
- vulnerabilidades conocidas críticas/altas: 0 abiertas en release;
- secretos confirmados: 0;
- findings estáticos críticos/altos: 0;
- tests flaky requeridos: 0;
- compatibilidad no documentada: 0;
- defectos Sev-1/Sev-2 abiertos: 0;
- evidencia inválida o no reproducible: 0;
- deuda sin dueño/fecha: 0.

La cobertura puede revelar ausencia de ejecución; no demuestra corrección. Por eso nunca compensa propiedades, mutación, aceptación o replay.

## 12. Toolchain y calificación de herramientas

Stack previsto:

- formato/lint: `rustfmt`, Clippy con warnings como error;
- tests: `cargo nextest`, doctests, `proptest`, snapshots `insta`;
- cobertura: `cargo llvm-cov`;
- mutación/fuzz: `cargo-mutants`, `cargo-fuzz`;
- dependencias: `cargo-deny`, GitHub dependency review;
- SAST: CodeQL para Rust y GitHub Actions;
- workflows: `actionlint` y `zizmor`;
- supply chain: SBOM CycloneDX/SPDX, GitHub artifact attestations y verificación de procedencia;
- automatización: `cargo xtask`, sin lógica crítica escondida en YAML de CI.

Cada herramienta posee versión o digest fijado, dueño, finalidad, input, output, tasa de falsos positivos conocida y método de actualización. Antes de hacer blocking una herramienta nueva se ejecuta en modo observación sobre al menos el corpus dorado y una release previa. Una actualización que cambie resultados exige revisión; el rollback consiste en restaurar el pin anterior, nunca desactivar el control.

## 13. Gestión de configuración, build y release

- Git es el registro de cambios; `main` es protegida y siempre liberable.
- Commits y tags de release se firman; no se permite force-push ni borrado de tags protegidos.
- Toda dependencia de producción está fijada por lockfile y política de fuentes/licencias.
- El build de release parte de tag inmutable, runner efímero y workflow revisado.
- Se genera SBOM, checksums, provenance/attestation y bundle de evidencia.
- El artefacto se verifica en un job separado antes de publicar.
- La promoción mueve el mismo digest; no recompila entre ambientes.
- La meta inicial es SLSA Build L2 y Source L3; la ruta a Build L3/Source L4 se audita antes de GA.

## 14. Seguridad y privacidad

El threat model se actualiza al cambiar agentes, permisos, protocolo, aislamiento, evidencia, telemetry o publicación. Controles mínimos:

- mínimo privilegio para tokens y workflows;
- acciones de terceros fijadas a SHA completo;
- OIDC y credenciales efímeras; ningún secreto en fixtures/evidencia;
- allowlist de red por corrida y denegación por defecto;
- redacción antes de persistir prompts, logs y outputs sensibles;
- límites de CPU, memoria, tiempo, disco, red y gasto;
- separación entre contenido no confiable y autoridad;
- validación de paths, symlinks, archivos especiales y archivos enormes;
- análisis de dependencias, licencias, secretos, SAST y workflows;
- respuesta y divulgación coordinada definidas en `SECURITY.md`.

## 15. No conformidades, defectos y CAPA

Severidad:

- `Sev-1`: compromiso, pérdida/corrupción de evidencia o release materialmente falsa;
- `Sev-2`: requisito crítico incumplido, aislamiento evadido o resultado no reproducible;
- `Sev-3`: función degradada con workaround;
- `Sev-4`: defecto menor/documental.

Sev-1/2 bloquean release. Una recurrencia, escape de gate o tendencia adversa abre CAPA (acción correctiva y preventiva) con causa raíz, alcance, acción, dueño, fecha y prueba de eficacia. Cerrar el issue sin comprobar eficacia no cierra CAPA.

## 16. Evidencia y auditoría

Cada gate produce un `GateRecord` que contiene:

- versión de plan, etapa y commit/tag;
- lista de controles aplicables y resultado;
- enlaces/digests de evidencia;
- requisitos/riesgos cubiertos;
- herramientas y versiones;
- waivers vigentes;
- autor/orquestador, revisores independientes, alcance, fecha y decisión;
- SHA congelado, comandos reproducidos, findings y veredicto individual de cada revisor.

Los bundles se conservan por la mayor de: vida de la release + 24 meses, obligación contractual o política de seguridad. Logs sensibles pueden tener retención menor, pero el manifiesto y su prueba de eliminación permanecen.

Una auditoría debe poder elegir una release y reconstruir: necesidad → requisito → arquitectura → cambio → prueba → evidencia → aprobación → artefacto exacto.

## 17. Excepciones y break-glass

Un waiver requiere ID, control afectado, justificación, alcance mínimo, riesgo residual, compensación, aprobador independiente y vencimiento máximo de 30 días. No se renueva sin CAPA. No existen waivers para evidencia falsificada, secreto confirmado, firma/atestado inválido ni vulnerabilidad crítica explotable en release.

Break-glass solo atiende un incidente activo. Se registra automáticamente, exige dos personas cuando sea técnicamente posible y produce revisión posterior en 24 horas.

## 18. Evolución frente al estado del arte

Mensualmente se revisan releases de toolchain y advisories; trimestralmente se realiza un radar técnico sobre investigación, agentes, evaluación, seguridad y supply chain. Una novedad no entra por moda: se formula hipótesis, benchmark reproducible, riesgos, coste, compatibilidad, datos de piloto y rollback. Las aceptadas generan ADR y actualización del plan; las rechazadas conservan la decisión para evitar reevaluación circular.

Las referencias externas tienen un dueño y fecha de revisión. Un estándar draft puede observarse, pero no reemplaza una baseline final sin decisión explícita.

## 19. Aprobación del plan

Gate G0 aprueba este plan cuando:

1. documentos y JSON pasan el validador;
2. todos los requisitos iniciales tienen realización, pruebas y gate;
3. roles nominales y repositorio remoto están definidos;
4. arquitectura y ADR iniciales están aceptados;
5. rulesets de GitHub se aplican y se prueban sin bypass ordinario;
6. decisiones abiertas bloqueantes se resuelven.

Hasta entonces el estado correcto es **propuesta**, no “plan aprobado”.
