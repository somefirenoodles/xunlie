# Política de seguridad

La seguridad es parte del contrato de Xunlie. Agradecemos los reportes responsables que permitan
corregir una vulnerabilidad antes de divulgarla públicamente.

## Versiones soportadas

Xunlie todavía no tiene una release estable. Durante la fase `0.x`, solo la última línea minor
recibe correcciones de seguridad.

| Versión | Soporte de seguridad |
|---|---|
| `0.1.x` | Sí |
| `< 0.1` | No |

No se garantiza soporte para commits anteriores, forks ni binarios distribuidos por terceros.

## Reportar una vulnerabilidad

Use exclusivamente
[GitHub Private Vulnerability Reporting](https://github.com/somefirenoodles/xunlie/security/advisories/new).
No abra un issue, una discussion ni una pull request pública con detalles explotables.

Incluya, cuando sea posible:

- componente, versión o commit afectado;
- descripción del impacto y escenario de amenaza;
- pasos mínimos para reproducir o una prueba de concepto segura;
- precondiciones y configuración relevante;
- mitigaciones conocidas;
- nombre o alias para el reconocimiento, si lo desea.

No adjunte secretos, datos personales, credenciales reales ni información de terceros. Un enlace
privado es preferible para artefactos grandes o sensibles.

## Qué puede esperar

El mantenedor procurará:

- acusar recibo en un máximo de 3 días laborables;
- entregar una evaluación inicial en un máximo de 7 días laborables;
- mantener una actualización al menos cada 14 días mientras el reporte permanezca abierto;
- coordinar corrección, release y divulgación con el reportante.

Estos tiempos son objetivos de respuesta de un proyecto mantenido actualmente por una sola
persona, no una garantía contractual. Si un reporte no recibe confirmación dentro del primer
plazo, puede reenviarlo mediante el mismo advisory privado.

## Alcance prioritario

Son especialmente relevantes:

- escape de sandbox, ejecución de comandos o path traversal;
- exfiltración de datos, secretos o contexto entre agentes;
- confusión de autoridad o escalada de privilegios;
- manipulación de evidencia, digests o veredictos;
- parsing inseguro de entradas no confiables;
- compromiso de dependencias, builds o artefactos de release.

Errores de uso, propuestas funcionales y problemas sin impacto de seguridad deben ir a
[Issues](https://github.com/somefirenoodles/xunlie/issues). Consulte también el
[modelo de amenazas](docs/security/THREAT-MODEL.md).

## Divulgación y puerto seguro

Solicitamos no divulgar el reporte hasta acordar una fecha o hasta que exista una corrección
razonable. El proyecto reconocerá al reportante si lo autoriza.

La investigación de buena fe debe limitarse a cuentas y datos propios, evitar degradar el
servicio o la cadena de suministro, y detenerse si encuentra información de terceros. El proyecto
no perseguirá a quien respete estas condiciones, reporte de forma privada y cumpla la legislación
aplicable.
