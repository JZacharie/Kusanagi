# 🕸️ Kusanagi (草薙)

**Kusanagi** est une plateforme de supervision et d'auto-remédiation pour Kubernetes, entièrement développée en **Rust**.

Inspiré par le Major Motoko Kusanagi (*Ghost in the Shell*), ce projet ne se contente pas d'observer le "flux" de votre cluster : il déploie des agents légers (Cyber-Brains) au sein de votre infrastructure pour diagnostiquer et agir directement sur les composants en temps réel.

## 🚀 Vision du Projet

Là où les outils traditionnels sont de simples miroirs, **Kusanagi** est un bras armé. Le projet repose sur trois piliers :

1. **Omniscience (Observabilité) :** Une vision granulaire des ressources K8s via l'API Server et des métriques de bas niveau.
2. **Agilité (Rust) :** Une empreinte mémoire minimale et une sécurité de type garantissant que l'outil de supervision ne devienne jamais une faille ou un poids pour le cluster.
3. **Action (Agents) :** Un système d'agents distribués capables d'exécuter des protocoles de remédiation (redémarrage intelligent, nettoyage de cache, ajustement de quotas) sans intervention humaine.

## 🛠 Architecture

* **Kusanagi Core :** Le cerveau central (Controller) écrit avec `kube-rs`. Il analyse l'état du cluster et orchestre les missions.
* **The Shell (Agents) :** Des binaires Rust ultra-légers déployés en `DaemonSet`. Ils agissent comme des sondes locales sur chaque nœud.
* **The Wired :** Une interface de monitoring temps réel (gRPC/Websocket) pour visualiser les flux de données.

## ⚡ Pourquoi Rust ?

* **Zéro-Cost Abstractions :** Pour monitorer des clusters massifs sans consommer de CPU inutile.
* **Memory Safety :** Crucial lorsque l'on déploie des agents avec des privilèges élevés sur des nœuds de production.
* **Single Binary :** Facilité de déploiement via des images Docker minimalistes (Distroless/Scratch).

> *"My shell may belong to the system, but my spirit is mine."*
