# Corrections Console Kusanagi - 2026-02-05

## ✅ Problèmes Corrigés

### 1. WebSocket - Fermeture immédiate (code 1006)
**Fichiers modifiés:**
- `static/js/core.js`
- `src/main.rs`

**Corrections:**
- Implémentation fonctionnelle du WebSocket backend avec actix-web-actors
- Backoff exponentiel pour reconnexions (1s → 2s → 4s → 8s → 16s)
- Message de connexion envoyé au client

### 2. Proxmox - Erreurs 503 et parsing JSON
**Fichiers modifiés:**
- `static/js/proxmox.js`
- `src/main.rs`

**Corrections:**
- Vérification `response.ok` avant parsing JSON côté frontend
- Activation des queries Proxmox via `proxmox_service`
- Retour de tableaux vides au lieu de 503 si service indisponible
- Vérifications DOM ajoutées

### 3. Home Assistant - Éléments DOM manquants
**Fichiers modifiés:**
- `static/js/homeassistant.js`

**Corrections:**
- Vérifications `if (element)` avant toute manipulation DOM
- Protection dans `renderStats()`, `renderSensors()`, `renderAutomations()`
- Gestion d'erreur améliorée dans `fetchAndRender()`

### 4. Dashboard - Données news manquantes
**Statut:** Déjà géré correctement dans le code existant

## 🚀 Déploiement

```bash
# Compilation
cargo build --release

# Déploiement rapide
./quick-deploy.sh

# Ou manuel
pkill -f kusanagi
./target/release/kusanagi
```

## 📊 Résultats Attendus

- ✅ WebSocket se connecte et reste connecté
- ✅ Pas d'erreurs de parsing JSON
- ✅ Pas d'erreurs DOM dans la console
- ✅ Proxmox affiche "No data" au lieu de crasher
- ✅ Reconnexions WebSocket espacées intelligemment

## 🔍 Vérification

```bash
# Logs backend
tail -f kusanagi.log

# Console browser
# Devrait voir: "✅ WebSocket connected" sans fermeture immédiate
```

## 📝 Notes Techniques

**WebSocket Backend:**
- Actor pattern avec actix-web-actors
- Gère ping/pong automatiquement
- Message de bienvenue JSON au client

**Proxmox Service:**
- Essaie API Proxmox (PROXMOX_URLS env var)
- Fallback sur commandes locales (qm, pct, pvecm)
- Détection processus QEMU/KVM et LXC
- Retourne toujours du JSON valide

**Frontend Resilience:**
- Toutes les manipulations DOM vérifiées
- Parsing JSON conditionnel sur response.ok
- Backoff exponentiel pour reconnexions
