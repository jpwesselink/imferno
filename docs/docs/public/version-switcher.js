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

      // Stable releases - one button per major, showing the highest version
      if (versions.stable?.length) {
        const byMajor = {};
        for (const s of versions.stable) {
          const m = s.version.match(/^v?(\d+)\.(\d+)\.(\d+)$/);
          if (!m) continue;
          const major = m[1];
          const num = Number(m[1]) * 1e6 + Number(m[2]) * 1e3 + Number(m[3]);
          if (!byMajor[major] || num > byMajor[major].num) {
            byMajor[major] = { ...s, num };
          }
        }
        const majors = Object.values(byMajor).sort((a, b) => b.num - a.num);
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
