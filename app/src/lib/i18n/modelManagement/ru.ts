import type { ModelManagementMessages } from './en';

const messages = {
  'feasibility.contextTooSmall': 'Размер контекста меньше суммы входных и выходных токенов',
  'feasibility.visionUnavailable':
    'По сохранённым метаданным распознавание изображений не поддерживается',
  'feasibility.overBudget': 'Расчётная стоимость превышает бюджет',
  'feasibility.unknownContext': 'Размер контекста неизвестен',
  'feasibility.unknownVision': 'Поддержка распознавания изображений неизвестна',
  'feasibility.unknownPrice': 'Цена не указана, равна нулю или некорректна',
  'feasibility.aria': 'Предварительная оценка пригодности моделей',
  'feasibility.title': 'Предпросмотр пригодности и стоимости моделей',
  'feasibility.advisory':
    'Оценка основана на сохранённых метаданных. Актуальные цены, квоты, ключи и доступность не проверены. Это только справочная информация; маршруты обработки задач не меняются.',
  'feasibility.input': 'Ожидаемые входные токены',
  'feasibility.cached': 'Кэшированные входные токены (часть входных)',
  'feasibility.output': 'Ожидаемые выходные токены',
  'feasibility.budget': 'Лимит стоимости (USD, необязательно)',
  'feasibility.budgetAria': 'Предел расчётной стоимости в долларах США',
  'feasibility.noLimit': 'Без ограничений',
  'feasibility.requireVision': 'Требуется зрение',
  'feasibility.invalidCached':
    'Число кэшированных входных токенов не должно превышать общее число входных токенов.',
  'feasibility.invalidCounts':
    'Введите неотрицательные целые числа токенов. Кэшированный ввод входит в общий объём ввода. При необходимости укажите неотрицательный предел стоимости.',
  'feasibility.fitCount': 'Соответствуют сохранённым метаданным',
  'feasibility.unknownCount': 'неизвестно',
  'feasibility.excludedCount': 'исключены заданными ограничениями',
  'feasibility.search': 'Поиск по результатам пригодности',
  'feasibility.filter': 'Фильтр по статусу пригодности',
  'feasibility.all': 'Все статусы',
  'feasibility.fits': 'Подходит по метаданным',
  'feasibility.unknown': 'Пригодность неизвестна',
  'feasibility.excluded': 'Исключено',
  'feasibility.fitDescription': 'Соответствуют сохранённым метаданным; доступность не проверена',
  'feasibility.excludedDescription': 'Исключены ограничениями',
  'feasibility.costUnknown': 'Стоимость неизвестна',
  'feasibility.empty': 'Нет сохранённых моделей для оценки.',
  'feasibility.noMatch': 'Нет моделей, соответствующих фильтрам.',
  'feasibility.showing': 'Показано',
  'feasibility.of': 'из',
  'feasibility.matching': 'подходящих записей моделей',
  'feasibility.more': 'Показать ещё модели',
  'fleet.reauthRequired':
    'Перед изменением адреса или способа аутентификации повторно введите ключ API в редакторе провайдера.',
  'fleet.groupSyncHintSafe':
    'По умолчанию сохранённые адреса провайдеров остаются без изменений. Проверьте и явно подтвердите изменения. Провайдеров с учётными данными необходимо обновлять через редактор, указав новый ключ.',
  'fleet.approveProviderSync': 'Подтвердить замену адресов и способов авторизации провайдеров',
  'fleet.previewProviderChanges': 'Предпросмотр изменений провайдеров',
  'fleet.providerNeedsKey': 'Требуется повторно ввести ключ',
  'fleet.presetHintSafe':
    'Импортируйте отсутствующие шаблоны моделей без изменения сохранённых метаданных. Обновление сведений о распознавании изображений, контексте и ценах необязательно и может заменить ручные правки. Маршруты и учётные данные не меняются.',
  'fleet.approveMetadataRefresh': 'Подтвердить замену сохранённых метаданных',
  'fleet.previewModelChanges': 'Предпросмотр изменений моделей',
  'fleet.refreshLabel': 'обновить сохранённые метаданные',
  'fleet.addLabel': 'добавить отсутствующий шаблон',
  'fleet.healthUnavailable': 'Сведения о состоянии набора моделей недоступны.',
  'fleet.selectProvidersError': 'Выберите одного или нескольких провайдеров.',
  'fleet.imageEndpointError':
    'Адреса API изображений должны использовать HTTPS или HTTP на петлевом адресе.',
  'fleet.chooseGroupError': 'Выберите группу провайдеров.',
  'fleet.modelImageEndpointError':
    'Введите адрес API изображений с HTTPS или HTTP на петлевом адресе.',
  'fleet.selectModelsError': 'Выберите одну или несколько моделей.',
  'fleet.contextError': 'Размер контекстного окна должен быть целым числом от 0 до 100 000 000.',
  'fleet.selectPatchError': 'Выберите хотя бы одно поле для обновления.',
  'fleet.noKeyUsage': 'В этом сеансе использование ключей инференса не зарегистрировано.',
  'fleet.selectProvider': 'Выбрать провайдера',
  'fleet.selected': 'выбрано',
  'fleet.groupImageAria': 'Базовый URL API изображений для группы',
  'fleet.rotationAria': 'Стратегия выбора среди нескольких токенов',
  'fleet.imageGroupAria': 'Группа провайдеров для адреса API изображений',
  'fleet.imageModelIdAria': 'Идентификатор модели для адреса API изображений',
  'fleet.modelImageAria': 'Базовый URL API изображений для модели',
  'fleet.searchAria': 'Фильтр шаблонов моделей',
  'fleet.selectModel': 'Выбрать модель',
  'fleet.visionLabel': 'распознавание изображений',
  'fleet.keepUnchanged': 'Оставить без изменений',
  'fleet.enabled': 'Включено',
  'fleet.disabled': 'Выключено',
  'fleet.leaveBlank': 'Оставьте пустым, чтобы сохранить значение',
  'fleet.bulkContextAria': 'Число токенов контекста для массового обновления',
  'fleet.prodexImageError':
    'Адреса API изображений Prodex должны использовать HTTPS или HTTP на петлевом адресе и не содержать учётных данных в URL.',
  'fleet.prodexModelImageError':
    'Адреса API изображений моделей Prodex должны использовать HTTPS или HTTP на петлевом адресе и не содержать учётных данных в URL.',
} satisfies ModelManagementMessages;

export default messages;
