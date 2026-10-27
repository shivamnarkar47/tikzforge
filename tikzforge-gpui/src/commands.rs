pub const LATEX_COMMANDS: &[&str] = &[
    "\\alpha", "\\beta", "\\gamma", "\\delta", "\\epsilon", "\\zeta", "\\eta", "\\theta",
    "\\iota", "\\kappa", "\\lambda", "\\mu", "\\nu", "\\xi", "\\pi", "\\rho", "\\sigma",
    "\\tau", "\\upsilon", "\\phi", "\\chi", "\\psi", "\\omega",
    "\\Gamma", "\\Delta", "\\Theta", "\\Lambda", "\\Pi", "\\Sigma", "\\Phi", "\\Psi", "\\Omega",
    "\\frac", "\\sqrt", "\\sum", "\\int", "\\prod", "\\lim", "\\log", "\\ln", "\\exp",
    "\\sin", "\\cos", "\\tan",
    "\\begin", "\\end", "\\documentclass", "\\usepackage", "\\newcommand",
    "\\title", "\\author", "\\date", "\\maketitle", "\\section", "\\subsection", "\\subsubsection",
    "\\textbf", "\\textit", "\\texttt", "\\emph", "\\underline",
    "\\label", "\\ref", "\\cite",
    "\\includegraphics", "\\caption", "\\centering",
    "\\tikz", "\\draw", "\\node", "\\path", "\\fill", "\\foreach", "\\tikzset",
    "\\begin{document}", "\\begin{tikzpicture}", "\\begin{figure}", "\\begin{equation}",
    "\\begin{align}", "\\begin{tabular}", "\\begin{itemize}", "\\begin{enumerate}",
    "\\end{document}", "\\end{tikzpicture}",
];

pub fn filter_commands(prefix: &str) -> Vec<&'static str> {
    if prefix.is_empty() { return vec![]; }
    let lower = prefix.to_lowercase();
    LATEX_COMMANDS.iter().filter(|c| c.to_lowercase().starts_with(&lower)).copied().take(20).collect()
}