

const tryMeCode = document.getElementsByClassName("tryme");
Array.prototype.forEach.call(
    tryMeCode,
    (container) => {
        const code = container.innerText;
        const previewUrl = `/editor/?source=${encodeURIComponent(code)}`;

        const link = document.createElement("a");
        link.href = previewUrl;
        const linkText = document.createTextNode("Open In Web Editor");
        link.appendChild(linkText);

        container.prepend(link)
    },
);
