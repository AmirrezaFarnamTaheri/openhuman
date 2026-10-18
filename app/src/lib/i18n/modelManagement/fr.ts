import type { ModelManagementMessages } from './en';

const messages = {
  'feasibility.contextTooSmall':
    'Le contexte est inférieur au total des jetons d’entrée et de sortie',
  'feasibility.visionUnavailable': 'Vision non prise en charge selon les métadonnées enregistrées',
  'feasibility.overBudget': 'Le coût estimé dépasse le budget',
  'feasibility.unknownContext': 'Taille du contexte inconnue',
  'feasibility.unknownVision': 'Capacité de vision inconnue',
  'feasibility.unknownPrice': 'Prix manquant, nul ou invalide',
  'feasibility.aria': 'Aperçu de l’adéquation du modèle',
  'feasibility.title': "Aperçu de l'adéquation et du coût des modèles",
  'feasibility.advisory':
    'Basé sur les métadonnées enregistrées, sans vérification en temps réel des prix, quotas, clés ou disponibilités. Ces indications ne modifient l’acheminement d’aucune tâche.',
  'feasibility.input': "Jetons d'entrée estimés",
  'feasibility.cached': "Jetons d'entrée en cache (inclus dans l'entrée)",
  'feasibility.output': 'Jetons de sortie estimés',
  'feasibility.budget': 'Plafond de coût (USD, facultatif)',
  'feasibility.budgetAria': 'Plafond de coût estimé en USD',
  'feasibility.noLimit': 'Sans limite',
  'feasibility.requireVision': 'Vision requise',
  'feasibility.invalidCached':
    'Les jetons d’entrée en cache ne peuvent pas dépasser le total des jetons d’entrée.',
  'feasibility.invalidCounts':
    'Saisissez des nombres entiers de jetons positifs ou nuls (le cache fait partie de l’entrée) et, si souhaité, un plafond de coût positif ou nul.',
  'feasibility.fitCount': 'Adéquation selon les métadonnées enregistrées',
  'feasibility.unknownCount': 'inconnus',
  'feasibility.excludedCount': 'exclus par les contraintes indiquées',
  'feasibility.search': 'Rechercher parmi les résultats',
  'feasibility.filter': "Filtrer par statut d'adéquation",
  'feasibility.all': 'Tous les statuts',
  'feasibility.fits': 'Compatible selon les métadonnées',
  'feasibility.unknown': 'Éligibilité inconnue',
  'feasibility.excluded': 'Exclu',
  'feasibility.fitDescription':
    'Correspond aux métadonnées enregistrées ; disponibilité non vérifiée',
  'feasibility.excludedDescription': 'Exclu par les contraintes',
  'feasibility.costUnknown': 'Coût inconnu',
  'feasibility.empty': 'Aucun modèle enregistré à évaluer.',
  'feasibility.noMatch': 'Aucun modèle ne correspond à ces filtres.',
  'feasibility.showing': 'Affichage',
  'feasibility.of': 'sur',
  'feasibility.matching': 'entrées de modèles correspondantes',
  'feasibility.more': 'Afficher plus de modèles',
  'fleet.reauthRequired':
    'Saisissez à nouveau la clé API dans l’éditeur du fournisseur avant de modifier son adresse ou son authentification.',
  'fleet.groupSyncHintSafe':
    'Les adresses enregistrées sont conservées par défaut. Examinez et approuvez explicitement les changements ; les fournisseurs dotés d’identifiants doivent être modifiés dans l’éditeur avec une nouvelle clé.',
  'fleet.approveProviderSync':
    "Autoriser le remplacement des adresses et méthodes d'authentification",
  'fleet.previewProviderChanges': 'Aperçu des modifications des fournisseurs',
  'fleet.providerNeedsKey': 'Saisie de la clé à nouveau requise',
  'fleet.presetHintSafe':
    'Importez les préréglages manquants sans modifier les métadonnées enregistrées. La mise à jour de la vision, du contexte et des prix existants est facultative et peut remplacer les modifications manuelles. Les routes et identifiants restent inchangés.',
  'fleet.approveMetadataRefresh': 'Autoriser le remplacement des métadonnées enregistrées',
  'fleet.previewModelChanges': 'Aperçu des modifications des modèles',
  'fleet.refreshLabel': 'actualiser les métadonnées enregistrées',
  'fleet.addLabel': 'ajouter le préréglage manquant',
  'fleet.healthUnavailable': 'L’état de l’ensemble des fournisseurs est indisponible.',
  'fleet.selectProvidersError': 'Sélectionnez un ou plusieurs fournisseurs.',
  'fleet.imageEndpointError': 'Les adresses d’images exigent HTTPS ou HTTP en boucle locale.',
  'fleet.chooseGroupError': 'Choisissez un groupe de fournisseurs.',
  'fleet.modelImageEndpointError':
    'Saisissez une adresse d’images en HTTPS ou HTTP en boucle locale.',
  'fleet.selectModelsError': 'Sélectionnez un ou plusieurs modèles.',
  'fleet.contextError':
    'La fenêtre de contexte doit être un entier compris entre 0 et 100 000 000.',
  'fleet.selectPatchError': 'Choisissez au moins un champ à modifier.',
  'fleet.noKeyUsage':
    'Aucune utilisation de clé d’inférence n’a été enregistrée dans cette session.',
  'fleet.selectProvider': 'Sélectionner un fournisseur',
  'fleet.selected': 'sélectionnés',
  'fleet.groupImageAria': 'URL de base de l’API d’images du groupe',
  'fleet.rotationAria': 'Stratégie de sélection entre plusieurs clés',
  'fleet.imageGroupAria': 'Groupe de fournisseurs de l’adresse d’images',
  'fleet.imageModelIdAria': 'Identifiant du modèle de l’adresse d’images',
  'fleet.modelImageAria': 'URL de base de l’API d’images du modèle',
  'fleet.searchAria': 'Filtrer les préréglages de modèles',
  'fleet.selectModel': 'Sélectionner un modèle',
  'fleet.visionLabel': 'vision',
  'fleet.keepUnchanged': 'Conserver sans modification',
  'fleet.enabled': 'Activé',
  'fleet.disabled': 'Désactivé',
  'fleet.leaveBlank': 'Laisser vide pour conserver',
  'fleet.bulkContextAria': 'Jetons de contexte à modifier en lot',
  'fleet.prodexImageError':
    'Les adresses d’images Prodex exigent HTTPS ou HTTP en boucle locale, sans identifiants dans l’URL.',
  'fleet.prodexModelImageError':
    'Les adresses d’images des modèles Prodex exigent HTTPS ou HTTP en boucle locale, sans identifiants dans l’URL.',
} satisfies ModelManagementMessages;

export default messages;
