# 🐳 DOCKER SETUP COMPLET

## Images Disponibles

### 1. Version Minimale
```bash
# Build
docker build -t kusanagi:clean .

# Run
docker run -d -p 8080:8080 --name kusanagi-clean kusanagi:clean

# Test
curl http://localhost:8080/health
```

### 2. Version Complète
```bash
# Build
cd kusanagi-hexagonal
docker build -t kusanagi:hexagonal .

# Run
docker run -d -p 8080:8080 --name kusanagi-hexagonal kusanagi:hexagonal

# Test
curl http://localhost:8080/health
```

## Caractéristiques

- **Multi-stage build** pour optimisation
- **Utilisateur non-root** pour sécurité
- **Health check** intégré
- **Image Debian slim** légère
- **SSL/TLS** support inclus
