# Deployment-Guide

## Übersicht

Dieser Guide beschreibt den Deployment-Prozess für die ACCI Framework Anwendung. Die Anwendung wird als Container-basierte Lösung bereitgestellt und kann sowohl in Kubernetes als auch als standalone Docker-Deployment ausgeführt werden.

## Voraussetzungen

- Docker 24.0 oder höher
- Docker Compose v2.0 oder höher
- Kubernetes 1.25 oder höher (für Kubernetes-Deployment)
- Helm 3.0 oder höher (für Kubernetes-Deployment)

## Container-Images

### Build-Prozess

```bash
# Production Build
docker build -t acci-framework:latest .

# Mit spezifischer Version
docker build -t acci-framework:1.0.0 .

# Multi-Platform Build
docker buildx build --platform linux/amd64,linux/arm64 -t acci-framework:latest .
```

### Image-Struktur

- Basis: Distroless
- Runtime: Rust Binary
- Frontend: Vorcompilierte WASM/JS Assets
- Konfiguration: Via Environment-Variablen und Config-Maps

## Docker Compose Deployment

### Basis-Setup

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

### Produktions-Setup

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

## Kubernetes Deployment

### Helm Chart Struktur

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

### Basis-Installation

```bash
# Repository hinzufügen
helm repo add acci-framework https://charts.acci-framework.dev
helm repo update

# Installation mit Standard-Konfiguration
helm install acci-framework acci-framework/acci-framework

# Installation mit Custom Values
helm install acci-framework acci-framework/acci-framework -f custom-values.yaml
```

### Beispiel values.yaml

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

## Monitoring & Observability

### Prometheus Metriken

Die Anwendung exponiert Metriken unter `/metrics` im Prometheus-Format:

- HTTP Request Metriken
- Business Metriken
- Runtime Metriken
- Custom Metriken

### Logging

- Strukturiertes JSON-Logging
- Log-Level konfigurierbar
- Correlation IDs für Request-Tracking
- Integration mit ELK/Loki

### Tracing

- OpenTelemetry Integration
- Jaeger/Zipkin kompatibel
- Distributed Tracing über Service-Grenzen
- Performance Monitoring

## Backup & Recovery

### Datenbank-Backups

```bash
# Backup erstellen
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  pg_dump -U acci acci > backup.sql

# Backup einspielen
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  psql -U acci acci < backup.sql
```

### Automatisierte Backups

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

## Disaster Recovery

### Recovery-Prozedur

1. Infrastruktur-Wiederherstellung
   ```bash
   helm install acci-framework acci-framework/acci-framework -f dr-values.yaml
   ```

2. Daten-Wiederherstellung
   ```bash
   # Datenbank wiederherstellen
   kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
     psql -U acci acci < backup.sql
   
   # Cache invalidieren
   kubectl exec -it $(kubectl get pods -l app=redis -o jsonpath='{.items[0].metadata.name}') -- \
     redis-cli FLUSHALL
   ```

3. Anwendungs-Validierung
   ```bash
   # Health Check
   curl -f https://api.acci-framework.dev/health
   
   # Metriken prüfen
   curl -f https://api.acci-framework.dev/metrics
   ```

## Performance-Optimierung

### Resource Limits

- CPU und Memory Limits basierend auf Lastprofil
- Horizontale und vertikale Skalierung
- Cache-Dimensionierung
- Connection Pool Tuning

### Netzwerk-Optimierung

- CDN für statische Assets
- Ingress-Konfiguration
- Service Mesh (optional)
- Load Balancing Strategien

## Security

### TLS-Konfiguration

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

### Network Policies

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

## Wartung

### Updates

```bash
# Chart-Update
helm repo update
helm upgrade acci-framework acci-framework/acci-framework

# Rollback wenn nötig
helm rollback acci-framework 1
```

### Debugging

```bash
# Logs anzeigen
kubectl logs -l app=acci-framework -f

# Pod Shell
kubectl exec -it $(kubectl get pods -l app=acci-framework -o jsonpath='{.items[0].metadata.name}') -- /bin/sh

# Port-Forwarding
kubectl port-forward svc/acci-framework 8080:8080
```

## Best Practices

### Deployment
- Immutable Infrastructure
- Zero-Downtime Deployments
- Canary Releases
- Blue-Green Deployments

### Monitoring
- Alert Rules definieren
- Dashboards erstellen
- SLO/SLI monitoring
- Error Budget tracking

### Security
- Regelmäßige Security Scans
- Vulnerability Management
- Access Control
- Audit Logging

### Backup
- Regelmäßige Backup-Validierung
- Off-site Backups
- Point-in-Time Recovery
- Backup Rotation 