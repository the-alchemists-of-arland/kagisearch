pub(crate) const HOST: &str = "https://kagi.com";

// Browser anti-detection script
pub(crate) const BROWSER_INIT_SCRIPT: &str = r#"
Object.defineProperty(navigator, "webdriver", { get: () => false });
Object.defineProperty(navigator, "plugins", { get: () => [1, 2, 3, 4, 5] });
Object.defineProperty(navigator, "languages", { get: () => ["en-US", "en", "zh-CN"] });
window.chrome = {
    runtime: {},
    loadTimes: function () {},
    csi: function () {},
    app: {}
};
if (typeof WebGLRenderingContext !== "undefined") {
    const getParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function (parameter) {
        if (parameter === 37445) return "Intel Inc.";
        if (parameter === 37446) return "Intel Iris OpenGL Engine";
        return getParameter.call(this, parameter);
    };
}
"#;

// Screen properties script
pub(crate) const SCREEN_INIT_SCRIPT: &str = r#"
Object.defineProperty(window.screen, "width", { get: () => 1920 });
Object.defineProperty(window.screen, "height", { get: () => 1080 });
Object.defineProperty(window.screen, "colorDepth", { get: () => 24 });
Object.defineProperty(window.screen, "pixelDepth", { get: () => 24 });
"#;
