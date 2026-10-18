import type { ModelManagementMessages } from './en';

const messages = {
  'feasibility.contextTooSmall': 'El contexto es menor que la suma de tokens de entrada y salida',
  'feasibility.visionUnavailable': 'Los metadatos guardados no indican compatibilidad con visión',
  'feasibility.overBudget': 'El costo estimado supera el presupuesto',
  'feasibility.unknownContext': 'Tamaño del contexto desconocido',
  'feasibility.unknownVision': 'Capacidad de visión desconocida',
  'feasibility.unknownPrice': 'Precio ausente, cero o no válido',
  'feasibility.aria': 'Vista previa de idoneidad del modelo',
  'feasibility.title': 'Vista previa de idoneidad y costo del modelo',
  'feasibility.advisory':
    'Se basa en los metadatos guardados, no en precios, cuotas, claves ni disponibilidad verificados en tiempo real. Es solo orientativo y no cambia la ruta de ninguna tarea.',
  'feasibility.input': 'Tokens de entrada estimados',
  'feasibility.cached': 'Tokens de entrada en caché (incluidos en entrada)',
  'feasibility.output': 'Tokens de salida estimados',
  'feasibility.budget': 'Límite de costo (USD, opcional)',
  'feasibility.budgetAria': 'Límite estimado de costo en USD',
  'feasibility.noLimit': 'Sin límite',
  'feasibility.requireVision': 'Requerir visión',
  'feasibility.invalidCached':
    'Los tokens de entrada en caché no pueden superar el total de tokens de entrada.',
  'feasibility.invalidCounts':
    'Introduce cantidades enteras no negativas de tokens (los de caché están incluidos en la entrada) y, si lo deseas, un límite de costo no negativo.',
  'feasibility.fitCount': 'Compatibilidad según metadatos guardados',
  'feasibility.unknownCount': 'desconocidos',
  'feasibility.excludedCount': 'excluidos por las restricciones indicadas',
  'feasibility.search': 'Buscar resultados de idoneidad',
  'feasibility.filter': 'Filtrar por estado de idoneidad',
  'feasibility.all': 'Todos los estados',
  'feasibility.fits': 'Compatible según metadatos',
  'feasibility.unknown': 'Elegibilidad desconocida',
  'feasibility.excluded': 'Excluido',
  'feasibility.fitDescription':
    'Coincide con los metadatos guardados; disponibilidad sin verificar',
  'feasibility.excludedDescription': 'Excluido por las restricciones',
  'feasibility.costUnknown': 'Costo desconocido',
  'feasibility.empty': 'No hay modelos guardados para evaluar.',
  'feasibility.noMatch': 'Ningún modelo coincide con estos filtros.',
  'feasibility.showing': 'Mostrando',
  'feasibility.of': 'de',
  'feasibility.matching': 'entradas de modelos coincidentes',
  'feasibility.more': 'Mostrar más modelos',
  'fleet.reauthRequired':
    'Vuelve a introducir la clave API en el editor del proveedor antes de cambiar su endpoint o autenticación.',
  'fleet.groupSyncHintSafe':
    'Los endpoints guardados se conservan de forma predeterminada. Revisa y aprueba expresamente los cambios; los proveedores con credenciales deben actualizarse desde el editor con una clave nueva.',
  'fleet.approveProviderSync':
    'Autorizar la sustitución de endpoints y autenticación de proveedores',
  'fleet.previewProviderChanges': 'Vista previa de cambios de proveedores',
  'fleet.providerNeedsKey': 'Requiere volver a introducir la clave',
  'fleet.presetHintSafe':
    'Importa los ajustes predefinidos que faltan sin cambiar los metadatos guardados. Actualizar visión, contexto y precios existentes es opcional y puede sustituir cambios manuales. No se modifican rutas ni credenciales.',
  'fleet.approveMetadataRefresh': 'Autorizar la sustitución de metadatos guardados',
  'fleet.previewModelChanges': 'Vista previa de cambios de modelos',
  'fleet.refreshLabel': 'actualizar los metadatos guardados',
  'fleet.addLabel': 'añadir el ajuste predefinido que falta',
  'fleet.healthUnavailable': 'No se puede consultar el estado del conjunto de proveedores.',
  'fleet.selectProvidersError': 'Selecciona uno o varios proveedores.',
  'fleet.imageEndpointError': 'Los endpoints de imágenes requieren HTTPS o HTTP de bucle local.',
  'fleet.chooseGroupError': 'Elige un grupo de proveedores.',
  'fleet.modelImageEndpointError':
    'Introduce un endpoint de imágenes con HTTPS o HTTP de bucle local.',
  'fleet.selectModelsError': 'Selecciona uno o varios modelos.',
  'fleet.contextError': 'La ventana de contexto debe ser un número entero entre 0 y 100.000.000.',
  'fleet.selectPatchError': 'Elige al menos un campo para actualizar.',
  'fleet.noKeyUsage': 'No se ha registrado uso de claves de inferencia en esta sesión.',
  'fleet.selectProvider': 'Seleccionar proveedor',
  'fleet.selected': 'seleccionados',
  'fleet.groupImageAria': 'URL base de la API de imágenes del grupo',
  'fleet.rotationAria': 'Estrategia de selección entre varias claves',
  'fleet.imageGroupAria': 'Grupo de proveedores del endpoint de imágenes',
  'fleet.imageModelIdAria': 'ID del modelo del endpoint de imágenes',
  'fleet.modelImageAria': 'URL base de la API de imágenes del modelo',
  'fleet.searchAria': 'Filtrar ajustes predefinidos de modelos',
  'fleet.selectModel': 'Seleccionar modelo',
  'fleet.visionLabel': 'visión',
  'fleet.keepUnchanged': 'Mantener sin cambios',
  'fleet.enabled': 'Activado',
  'fleet.disabled': 'Desactivado',
  'fleet.leaveBlank': 'Dejar vacío para conservar',
  'fleet.bulkContextAria': 'Tokens de contexto para actualización masiva',
  'fleet.prodexImageError':
    'Los endpoints de imágenes de Prodex requieren HTTPS o HTTP de bucle local, sin credenciales en la URL.',
  'fleet.prodexModelImageError':
    'Los endpoints de imágenes de modelos de Prodex requieren HTTPS o HTTP de bucle local, sin credenciales en la URL.',
} satisfies ModelManagementMessages;

export default messages;
