# Guide de Déploiement

## Vue d'ensemble

Ce guide décrit le processus de déploiement pour l'application ACCI Framework. L'application est fournie comme une solution basée sur conteneur et peut être exécutée aussi bien dans Kubernetes que comme un déploiement Docker autonome.

## Prérequis

- Docker 24.0 ou supérieur
- Docker Compose v2.0 ou supérieur
- Kubernetes 1.25 ou supérieur (pour le déploiement Kubernetes)
- Helm 3.0 ou supérieur (pour le déploiement Kubernetes)

## Images Container

### Processus de Build

```bash
# Build de Production
docker build -t acci-framework:latest .

# Avec version spécifique
docker build -t acci-framework:1.0.0 .

# Build Multi-Plateforme
docker buildx build --platform linux/amd64,linux/arm64 -t acci-framework:latest .
```

### Structure de l'Image

- Base : Distroless
- Runtime : Binaire Rust
- Frontend : Assets WASM/JS précompilés
- Configuration : Via variables d'environnement et Config-Maps

## Déploiement Docker Compose

### Configuration de Base

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    image: acci-framework:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://user:password@db:5432/acci
      - REDIS_URL=redis://cache:6379
      - LOG_LEVEL=info
    depends_on:
      - db
      - cache
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  db:
    image: postgres:15-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=acci

  cache:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### Configuration de Production

```yaml
# docker-compose.prod.yml
services:
  app:
    restart: always
    logging:
      driver: "json-file"
      options:
        max-size: "50m"
        max-file: "10"
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G

  db:
    restart: always
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G

  cache:
    restart: always
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 1G
        reservations:
          memory: 512M
```

## Déploiement Kubernetes

### Structure du Helm Chart

```
helm/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   └── NOTES.txt
└── charts/
    ├── postgresql
    └── redis
```

### Installation de Base

```bash
# Ajouter le repository
helm repo add acci-framework https://charts.acci-framework.dev
helm repo update

# Installation avec configuration standard
helm install acci-framework acci-framework/acci-framework

# Installation avec valeurs personnalisées
helm install acci-framework acci-framework/acci-framework -f custom-values.yaml
```

### Exemple values.yaml

```yaml
# values.yaml
image:
  repository: acci-framework
  tag: latest
  pullPolicy: IfNotPresent

replicaCount: 3

resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 1000m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
  targetMemoryUtilizationPercentage: 80

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.acci-framework.dev
      paths:
        - path: /
          pathType: Prefix

postgresql:
  enabled: true
  auth:
    username: acci
    database: acci
  primary:
    resources:
      limits:
        cpu: 4000m
        memory: 8Gi
      requests:
        cpu: 2000m
        memory: 4Gi

redis:
  enabled: true
  architecture: standalone
  auth:
    enabled: true
  master:
    resources:
      limits:
        cpu: 1000m
        memory: 1Gi
      requests:
        cpu: 500m
        memory: 512Mi
```

## Monitoring & Observabilité

### Métriques Prometheus

L'application expose des métriques sous `/metrics` au format Prometheus :

- Métriques de requêtes HTTP
- Métriques métier
- Métriques runtime
- Métriques personnalisées

### Journalisation

- Journalisation JSON structurée
- Niveau de log configurable
- IDs de corrélation pour le suivi des requêtes
- Intégration avec ELK/Loki

### Traçage

- Intégration OpenTelemetry
- Compatible Jaeger/Zipkin
- Traçage distribué à travers les services
- Surveillance des performances

## Sauvegarde & Récupération

### Sauvegardes de Base de Données

```bash
# Créer une sauvegarde
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  pg_dump -U acci acci > backup.sql

# Restaurer une sauvegarde
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  psql -U acci acci < backup.sql
```

### Sauvegardes Automatisées

```yaml
# cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: db-backup
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:15-alpine
            command:
            - /bin/sh
            - -c
            - pg_dump -h db -U acci acci | gzip > /backup/db-$(date +%Y%m%d).sql.gz
```

## Reprise après Sinistre

### Procédure de Récupération

1. Restauration de l'Infrastructure
   ```bash
   helm install acci-framework acci-framework/acci-framework -f dr-values.yaml
   ```

2. Restauration des Données
   ```bash
   # Restaurer la base de données
   kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
     psql -U acci acci < backup.sql
   
   # Invalider le cache
   kubectl exec -it $(kubectl get pods -l app=redis -o jsonpath='{.items[0].metadata.name}') -- \
     redis-cli FLUSHALL
   ```

3. Validation de l'Application
   ```bash
   # Vérification de santé
   curl -f https://api.acci-framework.dev/health
   
   # Vérifier les métriques
   curl -f https://api.acci-framework.dev/metrics
   ```

## Optimisation des Performances

### Limites de Ressources

- Limites CPU et mémoire basées sur le profil de charge
- Mise à l'échelle horizontale et verticale
- Dimensionnement du cache
- Réglage du pool de connexions

### Optimisation Réseau

- CDN pour les assets statiques
- Configuration Ingress
- Service Mesh (optionnel)
- Stratégies de répartition de charge

## Sécurité

### Configuration TLS

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.acci-framework.dev
    secretName: acci-framework-tls
```

### Politiques Réseau

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: acci-framework-policy
spec:
  podSelector:
    matchLabels:
      app: acci-framework
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
```

## Maintenance

### Mises à Jour

```bash
# Mise à jour du Chart
helm repo update
helm upgrade acci-framework acci-framework/acci-framework

# Rollback si nécessaire
helm rollback acci-framework 1
```

### Débogage

```bash
# Afficher les logs
kubectl logs -l app=acci-framework -f

# Shell dans le Pod
kubectl exec -it $(kubectl get pods -l app=acci-framework -o jsonpath='{.items[0].metadata.name}') -- /bin/sh

# Port-Forwarding
kubectl port-forward svc/acci-framework 8080:8080
```

## Meilleures Pratiques

### Déploiement
- Infrastructure Immuable
- Déploiements sans interruption
- Releases Canary
- Déploiements Blue-Green

### Surveillance
- Définir les règles d'alerte
- Créer des tableaux de bord
- Surveillance SLO/SLI
- Suivi du budget d'erreur

### Sécurité
- Scans de sécurité réguliers
- Gestion des vulnérabilités
- Contrôle d'accès
- Journalisation d'audit

### Sauvegarde
- Validation régulière des sauvegardes
- Sauvegardes hors site
- Récupération à un instant T
- Rotation des sauvegardes 