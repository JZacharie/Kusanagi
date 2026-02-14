# Guide de Migration Kusanagi v0.3.0 🚀

Ce guide détaille les changements architecturaux majeurs introduits dans la version 0.3.0 de Kusanagi. L'application a migré d'une architecture monolithique (avec des modules "legacy") vers une **Architecture Hexagonale** (Ports & Adapters) stricte.

## 🏗️ Nouvelle Architecture

Nous suivons désormais une séparation stricte en 4 couches :

1.  **Interface Layer** (`src/interfaces/`) : Entrée de l'application (HTTP, WebSocket).
2.  **Application Layer** (`src/application/`) : Orchestration, Use Cases.
3.  **Domain Layer** (`src/domain/`) : Logique métier pure, Entités, Ports.
4.  **Infrastructure Layer** (`src/infrastructure/`) : Implémentation technique (DB, K8s Client, Cache).

## 🚫 Ce qui a changé (Breaking Changes)

### 1. Suppression du dossier `src/legacy/`
Tous les anciens modules (`legacy/pods.rs`, `legacy/chat.rs`, etc.) ont été supprimés.
- **Remplacement** : Les fonctionnalités ont été réécrites sous forme de Use Cases dans `src/application/use_cases/`.
- **Impact** : Tout code dépendant de `crate::legacy::*` ne compilera plus.

### 2. Gestion des Routes
Les routes ne sont plus dispersées dans `main.rs`.
- **Nouveau** : `src/interfaces/http/routes.rs` centralise la configuration du routeur Axum.
- **Nouveau** : Les handlers sont organisés dans `src/interfaces/http/handlers/` par domaine (`core`, `business`, `k8s`, `monitoring`).

### 3. Injection de Dépendances
Nous utilisons `AppState` (`src/state.rs`) pour injecter les dépendances.
- Les repositories et services ne sont plus instanciés à la volée dans les handlers.
- Ils sont créés au démarrage dans `main.rs` et passés via `AppState`.

## 🛠️ Comment ajouter une nouvelle fonctionnalité ?

Ne créez plus de fichiers "fourre-tout". Suivez le flux hexagonal :

1.  **Domain** : Définissez vos entités (`domain/entities/`) et vos ports (`domain/ports/`).
2.  **Infrastructure** : Implémentez le port (`infrastructure/repositories/`) si nécessaire.
3.  **Application** : Créez un Use Case (`application/use_cases/`).
4.  **Interface** : Créez un Handler (`interfaces/http/handlers/`) qui appelle le Use Case, et enregistrez la route dans `routes.rs`.

## 🔍 Mapping des anciens modules

| Ancien Module (Legacy) | Nouveau Use Case / Service |
|------------------------|----------------------------|
| `legacy/pods.rs` | `ListPodsUseCase`, `GetPodDetailsUseCase` |
| `legacy/chat.rs` | `ChatService` + `ChatUseCase` |
| `legacy/system.rs` | `SystemService` (Domain Service) |
| `legacy/proxmox.rs` | `ProxmoxService` |
| `legacy/weather.rs` | `GetWeatherUseCase` |

---
*Généré automatiquement par Antigravity lors du Sprint 4.*
