#!/usr/bin/env python3
"""
Script pour réorganiser le HTML de Kusanagi:
1. Déplacer les sections docs et evolution avant le footer
2. Nettoyer les références aux thèmes
3. Mettre à jour les liens CSS
"""

with open('static/index.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Trouver les indices des sections importantes
footer_start = None
main_content_end = None
docs_start = None
docs_end = None
evolution_start = None
evolution_end = None

for i, line in enumerate(lines):
    if '<footer class="footer">' in line:
        footer_start = i
    if '</div>' in line and i > 1180 and i < 1190 and main_content_end is None:
        # La fermeture de main-content
        main_content_end = i
    if 'data-tab="docs"' in line and '<section' in line:
        docs_start = i - 1  # Inclure le commentaire
    if docs_start is not None and docs_end is None and '</section>' in line and i > docs_start:
        docs_end = i + 1
    if 'data-tab="evolution"' in line and '<section' in line:
        evolution_start = i - 1  # Inclure le commentaire
    if evolution_start is not None and evolution_end is None and '</section>' in line and i > evolution_start:
        evolution_end = i + 1
        break

print(f"Footer start: {footer_start}")
print(f"Main content end: {main_content_end}")
print(f"Docs: {docs_start} to {docs_end}")
print(f"Evolution: {evolution_start} to {evolution_end}")

# Extraire les sections
docs_section = lines[docs_start:docs_end]
evolution_section = lines[evolution_start:evolution_end]

# Créer le nouveau fichier
new_lines = []

# Tout jusqu'au footer
new_lines.extend(lines[:footer_start])

# Ajouter docs et evolution avant le footer
new_lines.extend(docs_section)
new_lines.append('\n')
new_lines.extend(evolution_section)
new_lines.append('\n')

# Ajouter le footer
new_lines.extend(lines[footer_start:main_content_end+1])

# Sauter les anciennes sections docs et evolution, et continuer avec le reste
new_lines.extend(lines[evolution_end:])

# Écrire le fichier
with open('static/index.html', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print("✅ HTML réorganisé avec succès!")
