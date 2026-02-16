{{/*
Expand the name of the chart.
*/}}
{{- define "north.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "north.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "north.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels.
*/}}
{{- define "north.labels" -}}
helm.sh/chart: {{ include "north.chart" . }}
{{ include "north.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels.
*/}}
{{- define "north.selectorLabels" -}}
app.kubernetes.io/name: {{ include "north.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "north.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "north.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
App image with tag.
*/}}
{{- define "north.image" -}}
{{- $tag := default (printf "v%s" .Chart.AppVersion) .Values.image.tag -}}
{{- printf "%s:%s" .Values.image.repository $tag }}
{{- end }}

{{/*
Secret name (chart-managed or existing).
*/}}
{{- define "north.secretName" -}}
{{- if .Values.existingSecret.name }}
{{- .Values.existingSecret.name }}
{{- else }}
{{- include "north.fullname" . }}
{{- end }}
{{- end }}

{{/*
Database host for init container wait checks.
*/}}
{{- define "north.dbHost" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" .Release.Name }}
{{- else }}
{{- .Values.externalDatabase.host }}
{{- end }}
{{- end }}

{{/*
Database port for init container wait checks.
*/}}
{{- define "north.dbPort" -}}
{{- if .Values.postgresql.enabled }}
{{- 5432 }}
{{- else }}
{{- .Values.externalDatabase.port | int }}
{{- end }}
{{- end }}

{{/*
DATABASE_URL construction.
Priority: secret.databaseUrl > subchart postgresql > externalDatabase
*/}}
{{- define "north.databaseUrl" -}}
{{- if .Values.secret.databaseUrl }}
{{- .Values.secret.databaseUrl }}
{{- else if .Values.postgresql.enabled }}
{{- $host := printf "%s-postgresql" .Release.Name }}
{{- $port := 5432 }}
{{- $db := .Values.postgresql.auth.database }}
{{- $user := .Values.postgresql.auth.username }}
{{- $pass := .Values.postgresql.auth.password }}
{{- printf "postgres://%s:%s@%s:%d/%s" $user $pass $host $port $db }}
{{- else }}
{{- $host := .Values.externalDatabase.host }}
{{- $port := .Values.externalDatabase.port | int }}
{{- $db := .Values.externalDatabase.database }}
{{- $user := .Values.externalDatabase.username }}
{{- $pass := .Values.externalDatabase.password }}
{{- printf "postgres://%s:%s@%s:%d/%s" $user $pass $host $port $db }}
{{- end }}
{{- end }}
