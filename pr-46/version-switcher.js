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

      // Stable releases - only show major versions (vX.0.0)
      if (versions.stable?.length) {
        const majors = versions.stable.filter((s) => {
          const m = s.version.match(/^v?(\d+)\.(\d+)\.(\d+)$/);
          return m && m[2] === "0" && m[3] === "0";
        });
        if (majors.length) {
          links.push(`<span class="label">Stable:</span>`);
          links.push(
            majors
              .map((s) => `<a href="${BASE}${s.path}">${s.version}</a>`)
              .join('<span class="separator">·</span>')
          );
        }
      }

      // Beta
      if (versions.beta) {
        links.push(
          `<span class="separator">|</span><span class="label">Beta:</span><a href="${BASE}${versions.beta.path}">${versions.beta.label}</a>`
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
