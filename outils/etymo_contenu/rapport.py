"""Diagnostics et rapport de validation."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field

ERREUR = "erreur"
AVERTISSEMENT = "avertissement"


@dataclass(frozen=True)
class Diagnostic:
    niveau: str
    regle: str
    fichier: str | None
    objet: str | None
    message: str


@dataclass
class Rapport:
    diagnostics: list[Diagnostic] = field(default_factory=list)
    fichiers_lus: int = 0

    def erreur(self, regle: str, message: str, fichier: str | None = None, objet: str | None = None) -> None:
        self.diagnostics.append(Diagnostic(ERREUR, regle, fichier, objet, message))

    def avertissement(self, regle: str, message: str, fichier: str | None = None, objet: str | None = None) -> None:
        self.diagnostics.append(Diagnostic(AVERTISSEMENT, regle, fichier, objet, message))

    @property
    def erreurs(self) -> list[Diagnostic]:
        return [d for d in self.diagnostics if d.niveau == ERREUR]

    @property
    def avertissements(self) -> list[Diagnostic]:
        return [d for d in self.diagnostics if d.niveau == AVERTISSEMENT]

    def regles(self, niveau: str | None = None) -> set[str]:
        return {d.regle for d in self.diagnostics if niveau is None or d.niveau == niveau}

    def trier(self) -> list[Diagnostic]:
        return sorted(
            self.diagnostics,
            key=lambda d: (d.niveau != ERREUR, d.fichier or "", d.objet or "", d.regle, d.message),
        )

    def en_texte(self) -> str:
        lignes = []
        for d in self.trier():
            lieu = d.fichier or "—"
            if d.objet:
                lieu += f" ({d.objet})"
            lignes.append(f"{d.niveau:<13} {lieu}  [{d.regle}]  {d.message}")
        nb_e, nb_a = len(self.erreurs), len(self.avertissements)
        lignes.append(
            f"{self.fichiers_lus} fichier{'s' if self.fichiers_lus > 1 else ''} lu{'s' if self.fichiers_lus > 1 else ''}, "
            f"{nb_e} erreur{'s' if nb_e > 1 else ''}, {nb_a} avertissement{'s' if nb_a > 1 else ''}."
        )
        return "\n".join(lignes)

    def en_json(self) -> str:
        return json.dumps(
            {
                "fichiers_lus": self.fichiers_lus,
                "erreurs": len(self.erreurs),
                "avertissements": len(self.avertissements),
                "diagnostics": [asdict(d) for d in self.trier()],
            },
            ensure_ascii=False,
            indent=2,
        )
