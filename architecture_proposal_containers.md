# Proposition d'Architecture EDA basée sur des Conteneurs Continus - Projet Lab

Cette architecture propose une alternative à l'approche Serverless (Lambda) en utilisant des **conteneurs persistants tournant en continu**. Cette solution s'appuie sur **Amazon ECS (Elastic Container Service) avec Fargate**, des microservices **Laravel 10** conteneurisés (Docker), et conserve la nature orientée événements (EDA) avec **Amazon SQS**, **Amazon S3** et **Redis**.

---

## 1. Schéma de l'Architecture Container-First

```mermaid
graph TD
    %% Frontend Layer
    subgraph Clients ["Couche Présentation"]
        Capacitor["App Mobile (Capacitor)"]
        Quasar["App Web (Quasar / Vue 3)"]
    end

    %% Routing Layer
    subgraph Routing ["Entrée & Load Balancing"]
        ALB["Application Load Balancer (ALB)"]
    end

    %% Microservices Container Layer (ECS)
    subgraph ECS_Cluster ["Cluster Amazon ECS (Fargate)"]
        subgraph Service_Auth ["Service Authentification"]
            Auth_Tasks["Tâches ECS Auth (Docker: PHP-FPM + Nginx / FrankenPHP)"]
        end

        subgraph Service_Core ["Service Cœur Métier"]
            Core_Tasks["Tâches ECS Core (Docker: PHP-FPM + Nginx)"]
        end

        subgraph Service_Notif ["Service Notifications & Workers"]
            Notif_Tasks["Tâches ECS Notifications"]
            Laravel_Workers["Workers Laravel (php artisan queue:work)"]
        end
    end

    %% Event Bus & Queue Layer
    subgraph EventBus ["Bus d'Événements & Files (EDA)"]
        SQS_Queues["Amazon SQS (Files d'attente dédiées)"]
        Redis_PubSub["Redis (Pub/Sub & Event Broadcasting)"]
    end

    %% Storage & Caching Layer
    subgraph Storage ["Stockage & Données"]
        ElastiCache_Redis["Amazon ElastiCache Redis (Cache & États Temps Réel)"]
        S3Bucket["Amazon S3 (Fichiers / Assets)"]
        RDS["Amazon RDS (MySQL/PostgreSQL)"]
    end

    %% Flow Connections
    Clients -->|HTTPS REST API| ALB
    ALB -->|Route /auth| Auth_Tasks
    ALB -->|Route /core| Core_Tasks

    %% Event Flows
    Core_Tasks -->|Push Event / Job| SQS_Queues
    SQS_Queues -->|Long Poll / Consume| Laravel_Workers
    Laravel_Workers -->|Trigger action| Notif_Tasks

    %% Live updates
    Core_Tasks -->|Broadcast event| Redis_PubSub
    Redis_PubSub -.->|WebSockets (Reverb / Soketi)| Clients
    
    %% Storage links
    ECS_Cluster -->|Cache & Session| ElastiCache_Redis
    ECS_Cluster -->|Uploads / Files| S3Bucket
    ECS_Cluster -->|Database| RDS
```

---

## 2. Changements Majeurs et Choix Technologiques

### A. Exécution des Microservices (ECS Fargate au lieu de Lambda)
*   **Dockerisation Standard** : Chaque microservice Laravel 10 est packagé dans une image Docker contenant un serveur web léger (comme Nginx + PHP-FPM, ou la solution moderne **FrankenPHP** qui excelle pour Laravel en conteneur).
*   **Amazon ECS (Fargate)** : 
    *   **Pourquoi ECS ?** Plus simple et moins coûteux à manager en termes d'exploitation que Kubernetes (EKS), tout en offrant un orchestrateur robuste pour des conteneurs qui tournent 24/7.
    *   **Pourquoi Fargate ?** C'est le mode serverless d'ECS. AWS gère les serveurs physiques sous-jacents, vous ne gérez que la taille de vos conteneurs (CPU/RAM).
*   **Application Load Balancer (ALB)** : Remplace l'API Gateway pour distribuer le trafic HTTP/HTTPS de manière performante et à faible latence vers les différents services ECS en fonction des chemins URL (ex: `/api/v1/auth/*` vers le Service Auth, `/api/v1/orders/*` vers le Service Core).

### B. Consommation des Événements (Workers Laravel continus)
*   **Workers Persistants (`php artisan queue:work`)** : Dans le modèle Lambda, SQS déclenchait directement une fonction. Avec des conteneurs continus, nous faisons tourner des conteneurs dédiés uniquement à l'écoute des files SQS (les workers Laravel standard).
*   **Avantage majeur** : Plus de problème de *Cold Start* (démarrage à froid). Le framework Laravel est déjà chargé en mémoire dans le conteneur et traite immédiatement les messages dès qu'ils arrivent dans SQS.

### C. Gestion du Temps Réel (WebSockets natifs)
*   Avec des conteneurs continus, vous pouvez héberger votre propre serveur de WebSockets (comme **Laravel Reverb**, introduit récemment, ou **Soketi**) directement dans un conteneur au sein du cluster ECS.
*   Ce conteneur WebSocket écoute le Pub/Sub de **Redis** et maintient des connexions TCP persistantes ouvertes avec les clients (Quasar & Capacitor) pour du push temps réel à très bas coût (contrairement à AWS API Gateway WebSocket qui facture au nombre de messages et à la durée de connexion).

---

## 3. Comparatif : Lambda (Serverless) vs ECS Fargate (Conteneurs Continus)

| Critère | Approche Lambda (Serverless) | Approche ECS Fargate (Conteneurs) |
| :--- | :--- | :--- |
| **Démarrage à froid (Cold Start)** | Réel défi pour PHP/Laravel, nécessite des optimisations. | Aucun. Les conteneurs tournent en continu et répondent instantanément. |
| **Coût à faible trafic** | Très faible (presque gratuit si aucun trafic). | Coût fixe minimum (le conteneur doit tourner 24/7 pour être disponible). |
| **Coût à fort trafic** | Peut devenir élevé (facturation à la requête). | Plus prédictible et souvent plus économique pour les charges de travail stables. |
| **Complexité de déploiement** | Bref + Serverless Framework / AWS SAM. | Dockerfiles standards + CI/CD (GitHub Actions vers AWS ECR et ECS). |
| **Base de données** | Risque de saturation des connexions (nécessite RDS Proxy). | Connexions persistantes réutilisées par les pools PHP-FPM classiques. |
| **WebSockets / Temps Réel** | Dépend d'AWS API Gateway WebSockets ou service externe. | Hébergement simple de Laravel Reverb ou Soketi dans le cluster. |

---

## 4. Recommandation pour le Projet Lab

Pour un **projet Lab**, l'architecture **ECS Fargate** présente un excellent équilibre :
1.  Elle utilise des standards du marché (Docker, Nginx/FrankenPHP, Workers Laravel natifs) très proches de vos environnements de développement locaux.
2.  Elle élimine la complexité de gestion liée au comportement éphémère de Lambda (temps d'exécution limité à 15 min, gestion des connexions DB, cold starts).
3.  Elle offre une scalabilité simple via ECS Auto Scaling (ajustement du nombre de conteneurs en fonction du CPU ou de la taille de la file SQS).
