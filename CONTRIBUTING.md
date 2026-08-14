# Contribuir a Xunlie

Gracias por ayudar a mejorar Xunlie. El proyecto acepta correcciones, documentación, casos de
prueba, propuestas de producto y mejoras de arquitectura. Al participar, acepta el
[Código de conducta](CODE_OF_CONDUCT.md).

## Antes de empezar

- Busque en los [issues abiertos](https://github.com/somefirenoodles/xunlie/issues) para evitar
  duplicados.
- Use el formulario correspondiente para proponer una funcionalidad, informar un defecto o
  plantear un cambio de arquitectura.
- Para preguntas de uso o ideas todavía no concretas, utilice
  [Discussions](https://github.com/somefirenoodles/xunlie/discussions).
- No publique vulnerabilidades. Siga el canal privado descrito en [SECURITY.md](SECURITY.md).

Un cambio de protocolo, esquema, frontera arquitectónica o dependencia de producción necesita
análisis explícito. Cuando altere una decisión estructural, proponga o actualice un ADR antes de
implementar el código.

## Preparar el entorno

Necesita Git, Python 3 y `rustup`. El repositorio selecciona automáticamente el toolchain fijado
en `rust-toolchain.toml`.

```console
git clone https://github.com/somefirenoodles/xunlie.git
cd xunlie
rustup show active-toolchain
cargo fetch --locked
cargo xtask quality
```

La [guía de desarrollo local](docs/development/LOCAL-DEVELOPMENT.md) contiene los requisitos,
comandos equivalentes a CI y soluciones para fallos frecuentes.

## Desarrollar un cambio

1. Parta de una rama corta y actualizada desde `main`.
2. Mantenga el cambio enfocado; no mezcle refactors ni actualizaciones de dependencias sin
   relación.
3. Añada pruebas para el comportamiento nuevo y para sus fallos relevantes.
4. Actualice documentación, requisitos, ADR y trazabilidad cuando corresponda.
5. Use mensajes de commit breves que describan el resultado del cambio.

El código Rust debe conservar las fronteras descritas en
[`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md). No reduzca un control,
omita un finding ni relaje una validación solo para obtener una ejecución verde.

## Verificar

Antes de abrir una pull request, ejecute:

```console
cargo xtask quality
```

Si cambió dependencias, ejecute también la comprobación de licencias, fuentes y advisories
indicada en la [guía local](docs/development/LOCAL-DEVELOPMENT.md). No se acepta que la
verificación modifique `Cargo.lock` de forma accidental.

## Abrir una pull request

Complete la plantilla y mantenga la PR lista para revisión:

- vincule el issue con `Closes #<número>` cuando corresponda;
- identifique requisitos `REQ-*`, riesgos `RISK-*` y ADR afectados, o indique que no aplican;
- explique el resultado observable y las decisiones no obvias;
- incluya comandos y resultados de verificación reproducibles;
- declare cambios de compatibilidad, seguridad, dependencias y migración;
- declare asistencia material de IA y cómo validó el resultado;
- confirme que no introdujo secretos, datos personales ni artefactos generados innecesarios.

Los checks automáticos son necesarios, pero no sustituyen la revisión de intención. El autor no
aprueba su propio cambio y debe resolver los comentarios críticos antes de integrar.

## Licencia de las contribuciones

Al enviar una contribución, acepta que se distribuya bajo la [licencia MIT](LICENSE) del
proyecto y confirma que tiene derecho a aportar ese contenido.
