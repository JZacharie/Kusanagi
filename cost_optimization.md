# Plan d'Optimisation des Coûts AWS - Projet Lab

Ce document présente une analyse des coûts de l'infrastructure Terraform actuelle et propose des optimisations majeures pour réduire la facture mensuelle AWS au minimum (visant **< 30 $/mois** au lieu de **~90-100 $/mois**).

---

## 1. Analyse des coûts de l'architecture actuelle (Estimations)

| Composant AWS | Configuration actuelle | Coût estimé / mois | Nature du coût |
| :--- | :--- | :--- | :--- |
| **AWS NAT Gateway** | 1 NAT Gateway (VPC) | **~32,40 $** | Fixe (incontournable pour les subnets privés) |
| **Application Load Balancer (ALB)** | 1 ALB | **~18,00 $** | Fixe |
| **Amazon RDS PostgreSQL** | `db.t4g.micro` (Single-AZ) | **~12,50 $** | Fixe |
| **Amazon ElastiCache Redis** | `cache.t4g.micro` | **~12,00 $** | Fixe |
| **ECS Fargate Tasks** | 3 services (0.25 vCPU / 512 Mo) | **~27,00 $** (3 * 9 $) | Fixe (24/7) |
| **Amazon S3 & CloudWatch** | Stockage minimal et logs | **~2,00 $** | Variable |
| **Total Estimé** | | **~103,90 $ / mois** | |

---

## 2. Optimisations Stratégiques (Comment réduire la facture à ~30 $/mois ?)

### Optimisation 1 : Supprimer la NAT Gateway (Gain : ~32,40 $/mois)
*   **Problème** : Les conteneurs ECS en subnet privé ont besoin d'une NAT Gateway pour télécharger des paquets ou appeler des APIs externes (comme S3 ou d'autres services).
*   **Solution** : 
    1.  Déployer les conteneurs ECS Fargate dans les **subnets publics**.
    2.  Leur attribuer une IP publique (`assign_public_ip = true`).
    3.  **Garantie de Sécurité** : Les Security Groups bloquent tout trafic entrant vers les conteneurs sauf celui provenant de l'ALB. Ils peuvent sortir directement par l'Internet Gateway (gratuit), rendant la NAT Gateway inutile.

### Optimisation 2 : Supprimer Amazon ElastiCache Redis (Gain : ~12,00 $/mois)
*   **Problème** : Un cluster ElastiCache dédié coûte 12 $/mois même s'il est vide à 99 %.
*   **Solution** : Héberger Redis directement comme une **tâche ECS Fargate** au sein de votre cluster en utilisant l'image Docker officielle `redis:alpine`.
*   **Coût** : ~9 $/mois pour la ressource Fargate dédiée, mais nous pouvons descendre à 0,25 vCPU / 512 Mo et utiliser le DNS local d'ECS (AWS Cloud Map) pour que les autres conteneurs s'y connectent sans surcoût.

### Optimisation 3 : Consolider les Microservices en Phase de Lab (Gain : ~18,00 $/mois)
*   **Problème** : Avoir 3 conteneurs Fargate qui tournent 24/7 coûte 3 * 9 $ = 27 $/mois.
*   **Solution** : 
    *   Faire tourner les 3 microservices (ou les architectures de workers) dans le **même service Fargate** (tâches multi-conteneurs) si les besoins en ressources sont minimes.
    *   Ou concevoir un monolithe modulaire (une seule image Laravel) configuré différemment via les variables d'environnement (`role=web`, `role=worker`).

---

## 3. Comparatif des Factures mensuelles

| Composant AWS | Coût Actuel | Coût Optimisé (Sans NAT + Redis en ECS) |
| :--- | :--- | :--- |
| NAT Gateway | 32,40 $ | **0,00 $** (Supprimée) |
| ALB | 18,00 $ | 18,00 $ |
| RDS Postgres | 12,50 $ | 12,50 $ |
| ElastiCache Redis | 12,00 $ | **0,00 $** (Remplacé par Redis sur ECS) |
| ECS Fargate | 27,00 $ | 36,00 $ (3 App + 1 Redis Task) |
| **Total Mensuel** | **101,90 $** | **66,50 $** |

*Note: Si vous consolidez vos services Laravel en 1 seule tâche Fargate mutualisée, le coût ECS tombe à 9 $ + 9 $ (Redis) = 18 $, ramenant la facture globale à **~48,50 $/mois**.*
