# ADR-001: Stratégie de Refactoring vers Architecture Hexagonale

**Status**: Accepted  
**Date**: 2026-02-03  
**Deciders**: Joseph Zacharie, AI Assistant  
**Consulted**: -  
**Informed**: Future contributors

---

## Context and Problem Statement

Kusanagi a démarré comme un projet monolithique avec une architecture "flat" (modules par feature sans séparation des concerns). Au fil du temps, cela a créé :

- **Couplage fort** entre la logique métier et les dépendances externes (Kubernetes client, DB)
- **Tests difficiles** nécessitant des mocks complexes ou des intégrations réelles
- **Code duplication** entre les handlers HTTP et les modules legacy
- **Dette technique** : 22 modules legacy sur 35 (63%)

Nous avons décidé d'adopter l'**Architecture Hexagonale** (Ports & Adapters) pour résoudre ces problèmes.

---

## Decision Drivers

1. **Testability** : Pouvoir tester la logique métier sans Kubernetes cluster
2. **Maintainability** : Isoler les changements d'infrastructure
3. **Flexibility** : Pouvoir switcher de PostgreSQL à autre chose facilement
4. **Team Growth** : Onboarding plus facile avec des boundaries clairs

---

## Considered Options

### Option 1: Layered Architecture (Traditional)
```
Controller → Service → Repository → DB
```

- **Pros**: Familier, simple à comprendre
- **Cons**: Le service dépend encore du repository (couplage fort)

### Option 2: Clean Architecture (Onion)
```
Entities → Use Cases → Interface Adapters → Frameworks
```

- **Pros**: Découplage total, testable
- **Cons**: Plus complexe, overkill pour notre scope actuel

### Option 3: Hexagonal Architecture (Chosen)
```
         ┌─────────────────────┐
         │     Application     │
         │  (Domain + Use Cases)│
         └──────────┬──────────┘
                    │ Ports (interfaces)
    ┌───────────────┼───────────────┐
    │               │               │
 Drivers        Domain         Driven
 (HTTP,        Logic          (DB, K8s)
 CLI,                           │
 Scheduler)                     │
    │                           │
    └───────────┬───────────────┘
                │
         Infrastructure
```

- **Pros**: Bon équilibre simplicité/découplage, excellente testability
- **Cons**: Courbe d'apprentissage initiale, boilerplate

---

## Decision

**Nous adoptons l'Architecture Hexagonale (Option 3)** avec les conventions suivantes :

### Structure de Projet

```
src/
├── domain/              # Coeur métier (pur Rust, no deps externes)
│   ├── entities/        # Structs métiers (Pod, Node, etc.)
│   ├── ports/           # Interfaces (traits) pour les dépendances
│   └── services/        # Logique métier complexe
├── application/         # Use cases (orchestration)
│   └── use_cases/       # Un fichier par use case
├── infrastructure/      # Implémentations des ports
│   └── repositories/    # Kubernetes, PostgreSQL, etc.
├── interfaces/          # Adaptateurs entrants (REST, CLI)
│   └── http/            # Handlers Actix-web
└── legacy/              # Code à migrer (diminuer progressivement)
```

### Règles Strictes

1. **Domain** ne dépend de **rien** d'autre (pas même `serde` si possible)
2. **Application** ne connaît que le **Domain**
3. **Infrastructure** implémente les **Ports** du Domain
4. **Interfaces** utilise **Application** et **Infrastructure**

### Dependency Rule
```rust
// ✅ Correct: Domain pur
pub struct Pod { name: String, status: PodStatus }

// ❌ Incorrect: Domain ne doit pas importer kube
use kube::api::Api;  // INTERDIT dans domain/
```

---

## Consequences

### Positive

- **Testability**: On peut mocker `KubernetesRepository` pour tester les use cases
- **Flexibility**: Switcher de `kube-rs` à un autre client sans toucher la logique métier
- **Clarity**: Boundaries explicites, difficile de "courir des raccourcis"
- **Parallel Work**: Plusieurs devs peuvent travailler sur domain/infra/interfaces en parallèle

### Negative

- **Boilerplate**: Nécessite plus de fichiers (traits + impl)
- **Learning Curve**: Nouveaux contributeurs doivent comprendre l'architecture
- **Refactoring Cost**: Migration des 22 modules legacy = ~6 mois de travail

### Neutral

- **Performance**: Léger overhead d'indirection (traits), négligeable

---

## Implementation Strategy

### Phase 1: Foundation (v1.2.0)
- Migrer les modules les plus utilisés : Cilium, Database, Health
- Établir les patterns et conventions
- Documenter via des ADRs et un guide de migration

### Phase 2: Expansion (v1.2.5-v1.3.0)
- Migrer 1-2 modules par sprint
- Prioriser par "Business Value / Complexity"
- Maintenir les deux systèmes (legacy + hexagonal) pendant la transition

### Phase 3: Completion (v1.4.0+)
- Derniers modules (Home Assistant, Weather - personal features)
- Suppression du dossier `legacy/`
- Célébration 🎉

---

## Migration Guide (Checklist)

Pour migrer un module existant vers l'architecture hexagonale :

### 1. Analyze
- [ ] Identifier les entités métier (structs)
- [ ] Identifier les dépendances externes (K8s, DB, etc.)
- [ ] Dessiner le flow de données

### 2. Domain
- [ ] Créer `src/domain/entities/{module}.rs`
- [ ] Définir les structs métier (sans dépendances externes)
- [ ] Créer `src/domain/ports/{module}_port.rs` avec les traits

### 3. Application
- [ ] Créer `src/application/use_cases/{module}_use_cases.rs`
- [ ] Implémenter les use cases (orchestration)
- [ ] Tests unitaires avec mocks

### 4. Infrastructure
- [ ] Créer `src/infrastructure/repositories/{module}_repository.rs`
- [ ] Implémenter les traits du Domain
- [ ] Tests d'intégration

### 5. Interfaces
- [ ] Créer `src/interfaces/http/{module}_handlers.rs`
- [ ] Mapper HTTP ↔ Use Cases
- [ ] OpenAPI documentation

### 6. Cleanup
- [ ] Marquer le module legacy comme deprecated
- [ ] Migrer les usages
- [ ] Supprimer après validation en production

---

## Examples

### Before (Legacy)
```rust
// src/legacy/pods.rs
use kube::Api;

pub async fn get_pod_status(client: Client, ns: &str, name: &str) -> Result<String, kube::Error> {
    let pods: Api<Pod> = Api::namespaced(client, ns);
    let pod = pods.get(name).await?;
    Ok(pod.status.unwrap().phase.unwrap())
}
```

**Problems**:
- Couplage fort avec `kube`
- Impossible à tester sans cluster
- Erreur technique (`kube::Error`) exposée

### After (Hexagonal)
```rust
// src/domain/entities/pod.rs
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub status: PodStatus,
}

pub enum PodStatus {
    Running,
    Pending,
    Failed(String),
}

// src/domain/ports/kubernetes_port.rs
#[async_trait]
pub trait KubernetesRepository: Send + Sync {
    async fn get_pod(&self, ns: &str, name: &str) -> Result<Pod, KusanagiError>;
}

// src/application/use_cases/get_pod_status.rs
pub struct GetPodStatusUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetPodStatusUseCase {
    pub async fn execute(&self, ns: &str, name: &str) -> Result<PodStatus, KusanagiError> {
        let pod = self.k8s_repo.get_pod(ns, name).await?;
        Ok(pod.status)
    }
}

// src/infrastructure/repositories/kubernetes_repository.rs
#[async_trait]
impl KubernetesRepository for KubernetesRepositoryImpl {
    async fn get_pod(&self, ns: &str, name: &str) -> Result<Pod, KusanagiError> {
        // Implémentation avec kube-rs
        // Mapping kube::Error → KusanagiError
    }
}
```

**Benefits**:
- Testable avec un mock de `KubernetesRepository`
- Erreurs métiers (`KusanagiError`) pas techniques
- Peut changer `kube-rs` sans toucher le use case

---

## Related Decisions

- ADR-002: Error Handling Strategy (`KusanagiError`)
- ADR-003: Testing Strategy (Unit vs Integration)

---

## References

- [Hexagonal Architecture by Alistair Cockburn](https://alistair.cockburn.us/hexagonal-architecture/)
- [The Clean Architecture by Uncle Bob](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust Hexagonal Architecture Example](https://github.com/thombergs/buckpal)

---

**Status**: Accepted  
**Next Review**: 2026-04-01 (fin Q1)
