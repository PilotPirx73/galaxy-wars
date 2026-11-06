# Orbit Duell

Ein rundenbasiertes Artillerie-Duell im Weltraum: Zwei Raumschiffe beschießen sich quer durch ein Sonnensystem, und jeder Planet lenkt die Schüsse mit seiner Schwerkraft ab. Inspiriert von den klassischen Gravitations-Duellen der Heimcomputer-Ära.

![Spielszene](docs/screenshot.png)

## Zwei Versionen

| | Browser-Version (`web/`) | Desktop-Version (`desktop/`) |
|---|---|---|
| Technik | Eine einzige HTML-Datei, kein Build nötig | Rust mit [macroquad](https://github.com/not-fl3/macroquad) |
| Läuft auf | Jedem Browser, auch Handy und Tablet | macOS, Windows, Linux |
| Starten | `web/index.html` öffnen | App starten (siehe unten) |

Beide Versionen können dasselbe.

## Spielelemente

- **Planeten:** zufällig verteilt. Größere und dichtere ziehen stärker.
- **Sonne** in der Mitte, mit besonders starker Schwerkraft.
- **Schwarze Löcher:** klein, aber sehr stark. Wahlweise versteckt: Sie zeigen sich erst, wenn ein Schuss nahe vorbeifliegt, und verschlucken, was ihnen zu nahe kommt.
- **Computer-Gegner** in drei Stufen. Er kennt versteckte Schwarze Löcher auch nicht.
- **Endlos-Modus:** Schüsse fliegen ohne Zeitlimit, bis sie treffen oder du sie beendest.
- **Alte Flugbahnen** bleiben gestrichelt sichtbar, damit man nachkorrigieren kann.
- **Sound:** Alle Klänge werden im Programm erzeugt. Das Fluggeräusch lässt sich separat abschalten.

## Steuerung

| Taste | Funktion |
|---|---|
| ← / → | Winkel |
| ↑ / ↓ | Geschwindigkeit |
| Shift | fein einstellen |
| Maus auf dem Spielfeld halten | zielen |
| Leertaste / Enter | Feuer, während des Flugs: Schuss beenden |
| N | neue Karte |
| M | Ton an/aus |
| F | Fluggeräusch an/aus |
| Esc | Menü |

## Desktop-Version bauen

Voraussetzung: [Rust](https://rustup.rs).

```sh
cd desktop
cargo run --release
```

**macOS-App** (`Orbit Duell.app`, per Doppelklick startbar):

```sh
cd desktop
./build-mac-app.sh
```

Ist zusätzlich das Intel-Ziel installiert (`rustup target add x86_64-apple-darwin`), entsteht eine Universal-App für Apple-Silicon- und Intel-Macs.

**Linux** braucht vorher einige Systembibliotheken, zum Beispiel unter Debian/Ubuntu:

```sh
sudo apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

### Fertige Downloads

Wird ein Versions-Tag gepusht (z. B. `v0.1.0`), baut GitHub Actions automatisch Versionen für macOS, Windows und Linux und hängt sie an ein Release an:

```sh
git tag v0.1.0
git push origin v0.1.0
```

Die Programme sind nicht bei Apple bzw. Microsoft signiert. Beim ersten Start warnt das System deshalb. Unter macOS hilft Rechtsklick → „Öffnen“, unter Windows „Weitere Informationen“ → „Trotzdem ausführen“.

## Browser-Version online spielen

Der Workflow `pages.yml` veröffentlicht `web/` auf GitHub Pages. Dafür einmalig im Repository unter *Settings → Pages* als Quelle **GitHub Actions** auswählen.

## Lizenz

[MIT](LICENSE)
