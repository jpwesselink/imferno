(() => {
  const BASE = "/imferno";

  async function init() {
    try {
      const res = await fetch(`${BASE}/versions.json`);
      if (!res.ok) return;
      const versions = await res.json();

      const banner = document.createElement("div");
      banner.className = "version-banner";

      const links = [];

      // Stable releases
      if (versions.stable?.length) {
        links.push(`<span class="label">Stable:</span>`);
        links.push(
          versions.stable
            .map((s) => `<a href="${BASE}${s.path}">${s.version}</a>`)
            .join('<span class="separator">·</span>')
        );
      }

      // Beta
      if (versions.beta) {
        links.push(
          `<span class="separator">|</span><span class="label">Beta:</span><a href="${BASE}${versions.beta.path}">${versions.beta.label}</a>`
        );
      }

      // PRs
      if (versions.prs?.length) {
        links.push(
          `<span class="separator">|</span><span class="label">PRs:</span>`
        );
        links.push(
          versions.prs
            .map((p) => `<a href="${BASE}${p.path}">${p.label}</a>`)
            .join('<span class="separator">·</span>')
        );
      }

      banner.innerHTML = links.join(" ");
      document.body.appendChild(banner);
    } catch {}
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
