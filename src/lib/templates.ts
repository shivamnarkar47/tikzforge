export interface Template {
  name: string;
  code: string;
}

export const TEMPLATES: Template[] = [
  {
    name: "Blank",
    code: "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\n\\end{document}\n",
  },
  {
    name: "Circle",
    code: "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}\n  \\draw (0,0) circle (1cm);\n\\end{tikzpicture}\n\\end{document}\n",
  },
  {
    name: "Grid",
    code: "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}\n  \\draw[step=0.5cm,gray,very thin] (0,0) grid (4,4);\n\\end{tikzpicture}\n\\end{document}\n",
  },
];

export function getTemplate(name: string): Template | undefined {
  return TEMPLATES.find((t) => t.name === name);
}