# Guía de Despliegue

## Visión General

Esta guía describe el proceso de despliegue para la aplicación ACCI Framework. La aplicación se proporciona como una solución basada en contenedores y puede ejecutarse tanto en Kubernetes como en un despliegue Docker independiente.

## Requisitos

- Docker 24.0 o superior
- Docker Compose v2.0 o superior
- Kubernetes 1.25 o superior (para despliegue en Kubernetes)
- Helm 3.0 o superior (para despliegue en Kubernetes)

## Imágenes de Contenedor

### Proceso de Build

```bash
# Build de Producción
docker build -t acci-framework:latest .

# Con versión específica
docker build -t acci-framework:1.0.0 .

# Build Multi-Plataforma
docker buildx build --platform linux/amd64,linux/arm64 -t acci-framework:latest .
```

### Estructura de la Imagen

- Base: Distroless
- Runtime: Binario Rust
- Frontend: Assets WASM/JS precompilados
- Configuración: A través de variables de entorno y Config-Maps

## Despliegue con Docker Compose

### Configuración Base

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

### Configuración de Producción

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

## Despliegue en Kubernetes

### Estructura del Helm Chart

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

### Instalación Base

```bash
# Agregar el repositorio
helm repo add acci-framework https://charts.acci-framework.dev
helm repo update

# Instalación con configuración estándar
helm install acci-framework acci-framework/acci-framework

# Instalación con valores personalizados
helm install acci-framework acci-framework/acci-framework -f custom-values.yaml
```

### Ejemplo values.yaml

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

## Monitoreo y Observabilidad

### Métricas Prometheus

La aplicación expone métricas bajo `/metrics` en formato Prometheus:

- Métricas de solicitudes HTTP
- Métricas de negocio
- Métricas de runtime
- Métricas personalizadas

### Registro

- Registro JSON estructurado
- Nivel de log configurable
- IDs de correlación para seguimiento de solicitudes
- Integración con ELK/Loki

### Trazado

- Integración OpenTelemetry
- Compatible con Jaeger/Zipkin
- Trazado distribuido a través de servicios
- Monitoreo de rendimiento

## Respaldo y Recuperación

### Respaldos de Base de Datos

```bash
# Crear respaldo
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  pg_dump -U acci acci > backup.sql

# Restaurar respaldo
kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
  psql -U acci acci < backup.sql
```

### Respaldos Automatizados

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

## Recuperación ante Desastres

### Procedimiento de Recuperación

1. Restauración de Infraestructura
   ```bash
   helm install acci-framework acci-framework/acci-framework -f dr-values.yaml
   ```

2. Restauración de Datos
   ```bash
   # Restaurar base de datos
   kubectl exec -it $(kubectl get pods -l app=postgresql -o jsonpath='{.items[0].metadata.name}') -- \
     psql -U acci acci < backup.sql
   
   # Invalidar caché
   kubectl exec -it $(kubectl get pods -l app=redis -o jsonpath='{.items[0].metadata.name}') -- \
     redis-cli FLUSHALL
   ```

3. Validación de Aplicación
   ```bash
   # Verificación de salud
   curl -f https://api.acci-framework.dev/health
   
   # Verificar métricas
   curl -f https://api.acci-framework.dev/metrics
   ```

## Optimización de Rendimiento

### Límites de Recursos

- Límites de CPU y memoria basados en perfil de carga
- Escalado horizontal y vertical
- Dimensionamiento de caché
- Ajuste de pool de conexiones

### Optimización de Red

- CDN para assets estáticos
- Configuración de Ingress
- Service Mesh (opcional)
- Estrategias de balanceo de carga

## Seguridad

### Configuración TLS

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

### Políticas de Red

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

## Mantenimiento

### Actualizaciones

```bash
# Actualización del Chart
helm repo update
helm upgrade acci-framework acci-framework/acci-framework

# Rollback si es necesario
helm rollback acci-framework 1
```

### Depuración

```bash
# Mostrar logs
kubectl logs -l app=acci-framework -f

# Shell en el Pod
kubectl exec -it $(kubectl get pods -l app=acci-framework -o jsonpath='{.items[0].metadata.name}') -- /bin/sh

# Port-Forwarding
kubectl port-forward svc/acci-framework 8080:8080
```

## Mejores Prácticas

### Despliegue
- Infraestructura Inmutable
- Despliegues sin interrupción
- Releases Canary
- Despliegues Blue-Green

### Monitoreo
- Definir reglas de alerta
- Crear dashboards
- Monitoreo de SLO/SLI
- Seguimiento de presupuesto de error

### Seguridad
- Escaneos de seguridad regulares
- Gestión de vulnerabilidades
- Control de acceso
- Registro de auditoría

### Respaldo
- Validación regular de respaldos
- Respaldos fuera del sitio
- Recuperación a un punto en el tiempo
- Rotación de respaldos 