#!/bin/sh
# Galaxy Wars 2026 – automatisches Update auf der Synology
# Holt das neueste Image von ghcr.io und startet den Container nur dann neu,
# wenn es wirklich eine neue Version gibt. Danach werden alte Images entfernt.
#
# Einrichtung: Systemsteuerung → Aufgabenplaner → Erstellen → Geplante Aufgabe →
# Benutzerdefiniertes Script, Benutzer „root“, täglich (z. B. 4:00 Uhr).

PROJEKT=/volume1/docker/galaxy-wars   # Ordner des Container-Manager-Projekts

export PATH="$PATH:/usr/local/bin"
cd "$PROJEKT" || { echo "Ordner $PROJEKT nicht gefunden"; exit 1; }

# je nach DSM-Version heißt der Befehl „docker compose“ oder „docker-compose“
if docker compose version >/dev/null 2>&1; then DC="docker compose"; else DC="docker-compose"; fi

$DC pull --quiet && $DC up -d
docker image prune -f >/dev/null
echo "Galaxy Wars: Update-Prüfung abgeschlossen ($(date))"
