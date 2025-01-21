# Udhëzuesi i Deployment

## Përmbledhje

Ky udhëzues përshkruan procesin e deployment për aplikacionin ACCI Framework. Aplikacioni ofrohet si një zgjidhje e bazuar në kontejner dhe mund të ekzekutohet si në Kubernetes ashtu edhe si një deployment i pavarur Docker.

## Kërkesat

- Docker 24.0 ose më i lartë
- Docker Compose v2.0 ose më i lartë
- Kubernetes 1.25 ose më i lartë (për deployment në Kubernetes)
- Helm 3.0 ose më i lartë (për deployment në Kubernetes)

## Imazhet e Kontejnerit

### Procesi i Build

```bash
# Production Build
docker build -t acci-framework:latest .

# Me version specifik
docker build -t acci-framework:1.0.0 .

# Multi-Platform Build
docker buildx build --platform linux/amd64,linux/arm64 -t acci-framework:latest .
```

### Struktura e Imazhit

- Baza: Distroless
- Runtime: Rust Binary
- Frontend: WASM/JS Assets të parakompiluar
- Konfigurimi: Përmes variablave të mjedisit dhe Config-Maps

## Docker Compose Deployment

### Setup Bazë

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

### Setup i Prodhimit

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

## Deployment në Kubernetes

### Struktura e Helm Chart

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

### Instalimi Bazë

```bash
# Shto repository
helm repo add acci-framework https://charts.acci-framework.dev
helm repo update

# Instalimi me konfigurim standard
helm install acci-framework acci-framework/acci-framework

# Instalimi me Values të personalizuara
helm install acci-framework acci-framework/acci-framework -f custom-values.yaml
```

### Shembull values.yaml

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

## Monitorimi & Vëzhgueshmëria

### Metrikat Prometheus

Aplikacioni ekspozon metrika nën `/metrics` në formatin Prometheus:

- Metrika të kërkesave HTTP
- Metrika të biznesit
- Metrika të runtime
- Metrika të personalizuara

### Logging

- Logging i strukturuar JSON
- Niveli i log-ut i konfigurueshëm
- ID-të e korrelacionit për gjurmimin e kërkesave
- Integrim me ELK/Loki

### Tracing

- Integrim OpenTelemetry
- Kompatibël me Jaeger/Zipkin
- Gjurmim i shpërndarë përmes kufijve të shërbimit
- Monitorim i performancës

## Backup & Recovery

### Backup-et e Bazës së të Dhënave

```bash
# Krijo backup
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  pg_dump -U acci acci > backup.sql

# Restauro backup
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  psql -U acci acci < backup.sql
```

### Backup-et e Automatizuara

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

## Rikuperimi nga Fatkeqësitë

### Procedura e Rikuperimit

1. Rikuperimi i Infrastrukturës
   ```bash
   helm install acci-framework acci-framework/acci-framework -f dr-values.yaml
   ```

2. Rikuperimi i të Dhënave
   ```bash
   # Restauro bazën e të dhënave
   kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
     psql -U acci acci < backup.sql
   
   # Invalido cache
   kubectl exec -it $(kubectl get pods -l app=redis -o jsonpath='{.items[0].metadata.name}') -- \
     redis-cli FLUSHALL
   ```

3. Validimi i Aplikacionit
   ```bash
   # Kontrolli i shëndetit
   curl -f https://api.acci-framework.dev/health
   
   # Kontrollo metrikat
   curl -f https://api.acci-framework.dev/metrics
   ```

## Optimizimi i Performancës

### Kufijtë e Burimeve

- Kufijtë e CPU dhe Memory bazuar në profilin e ngarkesës
- Shkallëzim horizontal dhe vertikal
- Dimensionimi i cache
- Rregullimi i Connection Pool

### Optimizimi i Rrjetit

- CDN për asetet statike
- Konfigurimi i Ingress
- Service Mesh (opsional)
- Strategjitë e Load Balancing

## Siguria

### Konfigurimi TLS

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

### Politikat e Rrjetit

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

## Mirëmbajtja

### Përditësimet

```bash
# Përditësimi i Chart
helm repo update
helm upgrade acci-framework acci-framework/acci-framework

# Rollback nëse nevojitet
helm rollback acci-framework 1
```

### Debugging

```bash
# Shiko logs
kubectl logs -l app=acci-framework -f

# Shell në Pod
kubectl exec -it $(kubectl get pods -l app=acci-framework -o jsonpath='{.items[0].metadata.name}') -- /bin/sh

# Port-Forwarding
kubectl port-forward svc/acci-framework 8080:8080
```

## Praktikat më të Mira

### Deployment
- Infrastrukturë e Pandryshueshme
- Deployments pa Ndërprerje
- Releases Canary
- Deployments Blue-Green

### Monitorimi
- Përcakto rregullat e alarmit
- Krijo dashboards
- Monitorimi i SLO/SLI
- Gjurmimi i Error Budget

### Siguria
- Skanimet e rregullta të sigurisë
- Menaxhimi i dobësive
- Kontrolli i aksesit
- Auditimi i log-eve

### Backup
- Validimi i rregullt i backup-eve
- Backup-et off-site
- Rikuperim në pikë-në-kohë
- Rotacioni i backup-eve 