# Plan de Actualización de Documentación

## Resumen
Este documento detalla las mejoras necesarias en la documentación de Jolt para cumplir con las guías de escritura modernas y mejorar la consistencia.

## Archivos Revisados
- README.md
- docs/specs.md
- docs/design.md
- docs/plan.md
- docs/specs-v7.md
- frontend-docs/es/index.html
- frontend-docs/en/index.html

## Problemas Detectados

### 1. README.md
- **Línea 3**: Título demasiado largo y descriptivo, no sigue formato de pregunta del usuario
- **Línea 5**: Frases largas (>20 palabras) que requieren relectura
- **Línea 34**: Uso de "ultrarrápido" (marketing language)
- **Línea 36**: Falta de resumen TL;DR al inicio
- **Línea 88**: Uso de emojis en títulos de sección
- **Línea 101**: Frases largas y complejas
- **Línea 103**: Uso de "jamás" (lenguaje excesivo)
- **Línea 143**: Falta de meta.contentType para documentos web

### 2. docs/specs.md
- **Línea 1**: Falta de meta.contentType
- **Línea 3**: Frases largas y complejas
- **Línea 5**: Uso de "Todos" (lenguaje excesivo)
- **Línea 114**: Uso de "retirados" (lenguaje negativo)
- **Línea 118**: Falta de resumen TL;DR
- **Línea 129**: Títulos con emojis
- **Línea 147**: Uso de "Auditoria" (título no descriptivo)

### 3. docs/design.md
- **Línea 1**: Falta de meta.contentType
- **Línea 3**: Frases largas y complejas
- **Línea 5**: Uso de "Todos" (lenguaje excesivo)
- **Línea 7**: Frases largas (>20 palabras)
- **Línea 113**: Falta de resumen TL;DR
- **Línea 119**: Títulos con emojis

### 4. docs/plan.md
- **Línea 1**: Falta de meta.contentType
- **Línea 3**: Frases largas y complejas
- **Línea 7**: Uso de "Todos" (lenguaje excesivo)
- **Línea 36**: Uso de "retirados" (lenguaje negativo)
- **Línea 42**: Falta de resumen TL;DR
- **Línea 55**: Títulos con emojis

### 5. docs/specs-v7.md
- **Línea 1**: Falta de meta.contentType
- **Línea 3**: Frases largas y complejas
- **Línea 5**: Uso de "Todos" (lenguaje excesivo)
- **Línea 6**: Uso de "ya no forma parte" (lenguaje negativo)
- **Línea 143**: Falta de resumen TL;DR
- **Línea 162**: Títulos con emojis

### 6. frontend-docs/es/index.html y frontend-docs/en/index.html
- **Línea 7**: Falta de meta.contentType
- **Línea 8**: Títulos con emojis
- **Línea 46**: Uso de "sin fricción" (marketing language)
- **Línea 47**: Falta de resumen TL;DR

## Plan de Acción

### Fase 1: README.md
1. Añadir resumen TL;DR al inicio
2. Simplificar títulos a formato de pregunta del usuario
3. Dividir frases largas en oraciones más cortas
4. Eliminar lenguaje de marketing y superlativos
5. Reemplazar emojis con texto descriptivo
6. Mejorar estructura con resumen por sección

### Fase 2: Documentación Técnica (docs/)
1. Añadir meta.contentType a todos los documentos
2. Simplificar títulos a formato de pregunta del usuario
3. Dividir frases largas y complejas
4. Eliminar lenguaje excesivo y negativo
5. Añadir resumen TL;DR a cada documento
6. Mejorar estructura con resumen por sección
7. Reemplazar emojis con texto descriptivo

### Fase 3: Documentación Web (frontend-docs/)
1. Añadir meta.contentType a los documentos HTML
2. Simplificar títulos y eliminar emojis
3. Eliminar lenguaje de marketing
4. Añadir resumen TL;DR a cada página
5. Mejorar estructura de navegación

### Fase 4: Actualización de Enlaces y Referencias
1. Actualizar todos los enlaces rotos o obsoletos
2. Verificar consistencia en terminología
3. Asegurar que todos los acrónimos estén definidos
4. Actualizar referencias a versiones antiguas

## Prioridades
1. **Alta**: README.md (página principal del proyecto)
2. **Media**: docs/specs.md, docs/design.md (documentación técnica principal)
3. **Media**: docs/plan.md (plan de desarrollo)
4. **Baja**: docs/specs-v7.md (especificaciones futuras)
5. **Baja**: frontend-docs/ (documentación web)

## Resultados Esperados
- Documentación más clara y concisa
- Mejor experiencia de lectura
- Consistencia en todo el corpus
- Cumplimiento con guías de escritura modernas
- Mejor indexación y búsqueda