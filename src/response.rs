// HTTP response creation and formatting functions
// This module handles building proper HTTP responses with security headers

use lambda_http::{Error, Response, Body};

/// Static HTML content served by our Lambda function
/// 
/// This uses the include_str! macro to embed the HTML file at compile time:
/// 1. The HTML file is read during compilation, not at runtime (very fast)
/// 2. The content is stored in the binary itself, not allocated on the heap
/// 3. It has a 'static lifetime, meaning it lives for the entire program duration
/// 4. This approach separates concerns - HTML in .html file, Rust logic in .rs file
/// 
/// The include_str! macro takes a file path relative to the current source file
/// and includes its contents as a string literal in the compiled binary.
/// 
/// Benefits of this approach:
/// - Better code organization (HTML separate from Rust code)
/// - Syntax highlighting for HTML in editors
/// - Easier to maintain and edit HTML content
/// - Still gets all the performance benefits of compile-time inclusion
const HTML_CONTENT: &str = include_str!("index.html");

/// Creates an HTTP response with HTML content and proper headers
/// 
/// This function encapsulates the logic for building HTTP responses that serve
/// our static HTML content. It demonstrates several important Rust and HTTP concepts:
/// 
/// ## Function Purpose:
/// 
/// This function takes our static HTML content and wraps it in a proper HTTP response
/// with the correct status code and headers. This separation of concerns makes the
/// code more modular and testable.
/// 
/// ## HTTP Response Structure:
/// 
/// An HTTP response consists of:
/// 1. **Status Code**: Indicates success (200) or various error conditions
/// 2. **Headers**: Metadata about the response (content type, caching, security)
/// 3. **Body**: The actual content being sent to the client
/// 
/// ## HTTP Status Codes Explained:
/// 
/// HTTP status codes are three-digit numbers that indicate the result of an HTTP request:
/// 
/// **2xx Success Codes:**
/// - **200 OK**: Request succeeded, response contains requested content
/// - **201 Created**: Request succeeded and a new resource was created
/// - **204 No Content**: Request succeeded but no content to return
/// 
/// **4xx Client Error Codes:**
/// - **400 Bad Request**: Request was malformed or invalid
/// - **401 Unauthorized**: Authentication required
/// - **403 Forbidden**: Server understood request but refuses to authorize it
/// - **404 Not Found**: Requested resource doesn't exist
/// - **405 Method Not Allowed**: HTTP method not supported for this resource
/// - **413 Request Entity Too Large**: Request body exceeds server limits
/// 
/// **5xx Server Error Codes:**
/// - **500 Internal Server Error**: Generic server error
/// - **502 Bad Gateway**: Invalid response from upstream server
/// - **503 Service Unavailable**: Server temporarily overloaded or down
/// 
/// For our static web server, we primarily use:
/// - **200 OK**: For successful HTML content delivery (this function)
/// - **405 Method Not Allowed**: For non-GET requests (implemented later)
/// - **500 Internal Server Error**: For unexpected server errors
/// 
/// ## Content-Type Header:
/// 
/// The "text/html" content type tells the browser:
/// - This is HTML content that should be parsed and rendered
/// - Use UTF-8 encoding (default for text/html)
/// - Apply HTML parsing rules and execute any embedded CSS/JavaScript
/// 
/// ## Error Handling:
/// 
/// The `map_err(Box::new)?` pattern converts response builder errors into
/// Lambda-compatible errors. This is necessary because different error types
/// need to be unified into a single error type that Lambda can handle.
/// 
/// ## Return Type:
/// 
/// Returns `Result<Response<Body>, Error>` where:
/// - `Ok(response)`: Successfully created HTTP response
/// - `Err(error)`: Failed to create response (rare, usually indicates programming error)
pub fn create_html_response() -> Result<Response<Body>, Error> {
    // Use the Response builder pattern to construct our HTTP response
    // This is a common Rust pattern that allows method chaining for configuration
    let response = Response::builder()
        // HTTP 200 OK Status Code:
        // This indicates that the request has succeeded and the server is returning
        // the requested content. For a static web server, this is the standard
        // response for successful GET requests to any valid path.
        // 
        // Why 200 OK for our use case:
        // - The client requested HTML content via HTTP GET
        // - Our server successfully processed the request
        // - We have content to return (our static HTML page)
        // - No errors occurred during processing
        .status(200)
        .header("content-type", "text/html")  // Tell browser this is HTML content
        // X-Frame-Options Security Header (Task 21 - Requirements 3.4)
        // 
        // The "DENY" directive prevents this page from being displayed in any frame,
        // iframe, embed, or object element, regardless of the site attempting to do so.
        // This is a critical security measure to prevent clickjacking attacks.
        // 
        // ## What is Clickjacking?
        // 
        // Clickjacking (also known as UI redressing) is an attack where a malicious website
        // tricks users into clicking on something different from what they perceive they
        // are clicking on. This is accomplished by loading the target page in a transparent
        // or opaque iframe and overlaying it with malicious content.
        // 
        // ## How Clickjacking Attacks Work:
        // 
        // 1. **Invisible Iframe**: The attacker creates a webpage that loads the target
        //    site (our Lambda function) in an invisible or transparent iframe.
        // 
        // 2. **Deceptive UI**: The attacker overlays their own UI elements (buttons, links,
        //    forms) on top of or around the iframe, making it appear as if the user is
        //    interacting with the attacker's site.
        // 
        // 3. **Misdirected Clicks**: When users think they're clicking on the attacker's
        //    UI elements, they're actually clicking on elements within the hidden iframe,
        //    potentially performing unintended actions on the target site.
        // 
        // 4. **Session Hijacking**: If the user is logged into the target site, their
        //    clicks could trigger authenticated actions without their knowledge.
        // 
        // ## Example Attack Scenarios:
        // 
        // - **Social Media**: Tricking users into "liking" posts or sharing content
        // - **Banking**: Causing users to transfer money or change account settings
        // - **E-commerce**: Making users purchase items or change shipping addresses
        // - **Admin Panels**: Tricking administrators into changing system settings
        // 
        // ## How X-Frame-Options: DENY Protects Us:
        // 
        // - **Complete Frame Prevention**: The "DENY" value prevents the page from being
        //   displayed in ANY frame, iframe, embed, or object element, regardless of the
        //   origin of the framing page.
        // 
        // - **Browser Enforcement**: Modern browsers will refuse to load the page in a
        //   frame and may display an error message or blank content instead.
        // 
        // - **Universal Protection**: Unlike "SAMEORIGIN" (which allows framing from the
        //   same origin), "DENY" provides complete protection against all framing attempts.
        // 
        // - **Legacy Browser Support**: X-Frame-Options is supported by older browsers
        //   that may not support the newer Content Security Policy frame-ancestors directive.
        // 
        // ## Alternative X-Frame-Options Values:
        // 
        // - **DENY**: Prevents framing from any origin (most secure, what we use)
        // - **SAMEORIGIN**: Allows framing only from the same origin as the page
        // - **ALLOW-FROM uri**: Allows framing only from the specified URI (deprecated)
        // 
        // ## Why DENY is Appropriate for Our Static Server:
        // 
        // 1. **No Legitimate Framing Use Case**: Our static HTML page doesn't need to be
        //    embedded in other sites, so there's no functional reason to allow framing.
        // 
        // 2. **Maximum Security**: DENY provides the strongest protection against
        //    clickjacking attacks with no functional trade-offs for our use case.
        // 
        // 3. **Simple Implementation**: DENY is straightforward and doesn't require
        //    maintaining a list of allowed origins like ALLOW-FROM would.
        // 
        // 4. **Future-Proof**: Even if the content changes in the future, DENY ensures
        //    that clickjacking protection remains in place.
        // 
        // ## Modern Alternative: Content Security Policy
        // 
        // While X-Frame-Options is still widely used and supported, the modern approach
        // is to use Content Security Policy (CSP) with the frame-ancestors directive:
        // 
        // ```
        // Content-Security-Policy: frame-ancestors 'none'
        // ```
        // 
        // However, X-Frame-Options provides better compatibility with older browsers,
        // and many security-conscious applications include both headers for maximum
        // protection (defense in depth).
        // 
        // ## Implementation Notes:
        // 
        // - The header name is case-insensitive, but we use standard capitalization
        // - The "DENY" value is case-insensitive but conventionally uppercase
        // - This header should be included on ALL responses that could be framed
        // - Some browsers may show a console warning when framing is blocked
        .header("x-frame-options", "DENY")  // Prevent clickjacking attacks
        // X-Content-Type-Options Security Header (Task 20 - Requirements 3.4)
        // 
        // The "nosniff" directive prevents browsers from MIME type sniffing, which is a
        // security vulnerability where browsers try to guess the content type of a response
        // based on its content rather than trusting the Content-Type header.
        // 
        // ## What is MIME Type Sniffing?
        // 
        // MIME type sniffing (also called content sniffing) is when browsers examine the
        // actual content of a response to determine its type, rather than relying solely
        // on the Content-Type header sent by the server. While this was originally designed
        // to help with misconfigured servers, it creates security vulnerabilities.
        // 
        // ## Security Risks of MIME Type Sniffing:
        // 
        // 1. **Content Type Confusion**: An attacker could upload a file that appears to be
        //    an image but contains JavaScript code. Without nosniff, the browser might
        //    execute the JavaScript instead of displaying it as an image.
        // 
        // 2. **Cross-Site Scripting (XSS)**: Malicious content could be interpreted as
        //    executable code (HTML/JavaScript) even when served with a safe Content-Type
        //    like "text/plain" or "image/jpeg".
        // 
        // 3. **File Upload Attacks**: User-uploaded files could be executed as scripts
        //    if the browser sniffs them as executable content, bypassing server-side
        //    content type restrictions.
        // 
        // 4. **Polyglot Attacks**: Specially crafted files that are valid in multiple
        //    formats (e.g., both a valid image and valid JavaScript) could be executed
        //    as scripts when intended to be displayed as images.
        // 
        // ## How X-Content-Type-Options: nosniff Protects Us:
        // 
        // - **Enforces Content-Type**: Browsers must respect the Content-Type header
        //   and not attempt to guess the content type from the response body.
        // 
        // - **Prevents Script Execution**: Files served with non-executable Content-Types
        //   (like "text/plain" or "image/jpeg") cannot be executed as JavaScript, even
        //   if they contain script-like content.
        // 
        // - **Blocks Stylesheet Loading**: CSS files must be served with "text/css"
        //   Content-Type to be loaded as stylesheets when nosniff is enabled.
        // 
        // - **Reduces Attack Surface**: Eliminates an entire class of content-type
        //   confusion attacks that rely on browser sniffing behavior.
        // 
        // ## Why This Matters for Our Static Server:
        // 
        // Even though our Lambda function only serves static HTML content from a string
        // constant, the X-Content-Type-Options header is still important because:
        // 
        // 1. **Defense in Depth**: Security best practice to include all relevant
        //    security headers, even if the current implementation doesn't strictly need them.
        // 
        // 2. **Future Extensibility**: If the server is later extended to serve user-uploaded
        //    content or dynamic content, this header provides protection.
        // 
        // 3. **Compliance**: Many security standards and frameworks require this header
        //    to be present on all HTTP responses.
        // 
        // 4. **Browser Compatibility**: Some security scanners and browser security
        //    features expect this header to be present.
        // 
        // 5. **Consistent Security Posture**: Including this header demonstrates a
        //    commitment to security best practices and helps prevent future vulnerabilities.
        // 
        // ## Implementation Notes:
        // 
        // - The "nosniff" value is the only valid value for X-Content-Type-Options
        // - This header should be included on ALL responses, not just HTML responses
        // - The header is case-insensitive, but we use the standard capitalization
        // - Modern browsers (IE8+, Chrome, Firefox, Safari) all support this header
        .header("x-content-type-options", "nosniff")  // Prevent MIME type sniffing attacks
        // Content-Security-Policy Security Header (Task 22 - Requirements 3.4)
        // 
        // Content Security Policy (CSP) is a security standard that helps prevent
        // Cross-Site Scripting (XSS), data injection attacks, and other code injection
        // attacks by controlling which resources the browser is allowed to load.
        // 
        // ## What is Content Security Policy?
        // 
        // CSP is a browser security feature that allows web servers to declare which
        // dynamic resources are allowed to be loaded by a web page. It works by
        // defining a whitelist of trusted sources for various types of content
        // (scripts, stylesheets, images, fonts, etc.).
        // 
        // ## How CSP Prevents Attacks:
        // 
        // 1. **Cross-Site Scripting (XSS) Prevention**: By restricting where scripts
        //    can be loaded from, CSP prevents malicious scripts injected by attackers
        //    from executing, even if they bypass input validation.
        // 
        // 2. **Data Injection Protection**: CSP prevents attackers from injecting
        //    malicious content (like unauthorized stylesheets or images) that could
        //    be used for phishing or data exfiltration.
        // 
        // 3. **Clickjacking Mitigation**: The frame-ancestors directive (similar to
        //    X-Frame-Options) prevents the page from being embedded in malicious frames.
        // 
        // 4. **Mixed Content Prevention**: CSP can enforce HTTPS-only resource loading,
        //    preventing downgrade attacks on secure pages.
        // 
        // ## Our CSP Policy Breakdown:
        // 
        // **default-src 'self'**: This is the fallback directive that applies to all
        // resource types not explicitly covered by other directives. 'self' means
        // resources can only be loaded from the same origin (same protocol, domain, and port).
        // 
        // **script-src 'self'**: Only allow JavaScript to be loaded from the same origin.
        // This prevents inline scripts and external scripts from untrusted domains.
        // 
        // **style-src 'self' 'unsafe-inline'**: Allow stylesheets from the same origin
        // and also allow inline styles. We include 'unsafe-inline' because our HTML
        // contains inline CSS for simplicity. In a production application, you'd
        // typically move CSS to external files and remove 'unsafe-inline'.
        // 
        // **img-src 'self' data:**: Allow images from the same origin and also data: URLs
        // (base64-encoded images). This is common for small icons and embedded images.
        // 
        // **font-src 'self'**: Only allow fonts to be loaded from the same origin.
        // 
        // **connect-src 'self'**: Only allow AJAX requests, WebSocket connections, and
        // other network connections to the same origin.
        // 
        // **frame-ancestors 'none'**: Prevent this page from being embedded in any
        // frame, iframe, or object. This is equivalent to X-Frame-Options: DENY but
        // is the modern CSP approach.
        // 
        // **base-uri 'self'**: Only allow the HTML <base> element to use URLs from
        // the same origin, preventing base tag injection attacks.
        // 
        // **form-action 'self'**: Only allow forms to submit to the same origin,
        // preventing form hijacking attacks.
        // 
        // ## Why This Policy is Appropriate for Our Static Server:
        // 
        // 1. **Minimal Attack Surface**: Our static HTML page doesn't need to load
        //    external resources, so restricting everything to 'self' is appropriate.
        // 
        // 2. **Inline CSS Support**: We include 'unsafe-inline' for styles because
        //    our HTML contains embedded CSS for simplicity and self-containment.
        // 
        // 3. **Future-Proof**: If the static content is later extended with images,
        //    fonts, or other resources, this policy provides a secure foundation.
        // 
        // 4. **Defense in Depth**: Even though our current content is static and
        //    trusted, CSP provides protection against future vulnerabilities.
        // 
        // ## CSP vs Other Security Headers:
        // 
        // - **CSP frame-ancestors vs X-Frame-Options**: CSP is more modern and flexible,
        //   but X-Frame-Options has better legacy browser support. We include both.
        // 
        // - **CSP vs X-Content-Type-Options**: These serve different purposes and
        //   should be used together for comprehensive protection.
        // 
        // ## CSP Reporting and Monitoring:
        // 
        // In production applications, you can add report-uri or report-to directives
        // to receive reports when CSP violations occur. This helps detect attacks
        // and identify legitimate resources that need to be whitelisted.
        // 
        // ## Implementation Notes:
        // 
        // - CSP directives are separated by semicolons
        // - Source values are separated by spaces within each directive
        // - 'self' must be quoted (it's a keyword, not a URL)
        // - The policy should be as restrictive as possible while still allowing
        //   legitimate functionality
        .header("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'")  // Restrict resource loading
        // X-XSS-Protection Security Header (Task 23 - Requirements 3.4)
        // 
        // The X-XSS-Protection header enables the browser's built-in Cross-Site Scripting (XSS)
        // filter and configures how it should behave when XSS attacks are detected.
        // 
        // ## What is Cross-Site Scripting (XSS)?
        // 
        // Cross-Site Scripting is a security vulnerability where attackers inject malicious
        // scripts into web pages viewed by other users. These scripts execute in the victim's
        // browser with the same privileges as legitimate scripts from the website.
        // 
        // ## Types of XSS Attacks:
        // 
        // 1. **Reflected XSS**: Malicious script is reflected off a web server, typically
        //    through URL parameters or form inputs that are immediately displayed back
        //    to the user without proper sanitization.
        // 
        // 2. **Stored XSS**: Malicious script is permanently stored on the target server
        //    (in databases, message forums, comment fields, etc.) and served to users
        //    when they access the stored content.
        // 
        // 3. **DOM-based XSS**: The vulnerability exists in client-side JavaScript code
        //    that processes user input and dynamically updates the DOM without proper
        //    validation or encoding.
        // 
        // ## How XSS Attacks Work:
        // 
        // 1. **Script Injection**: Attacker finds a way to inject JavaScript code into
        //    a web page (through forms, URL parameters, or stored content).
        // 
        // 2. **Victim Visits Page**: When a victim visits the compromised page, their
        //    browser executes the malicious script as if it were legitimate content.
        // 
        // 3. **Malicious Actions**: The script can steal cookies, session tokens, personal
        //    information, redirect users to malicious sites, or perform actions on
        //    behalf of the victim.
        // 
        // ## X-XSS-Protection Header Values:
        // 
        // - **"0"**: Disables XSS filtering entirely (not recommended)
        // - **"1"**: Enables XSS filtering (sanitizes the page if attack detected)
        // - **"1; mode=block"**: Enables XSS filtering and blocks the entire page if
        //   attack detected (most secure option, what we use)
        // - **"1; report=<reporting-uri>"**: Enables filtering and sends violation
        //   reports to the specified URI
        // 
        // ## Why "1; mode=block" is Most Secure:
        // 
        // - **Complete Protection**: When XSS is detected, the browser blocks the entire
        //   page from loading, preventing any part of the attack from executing.
        // 
        // - **No Partial Rendering**: Unlike the default "1" mode which tries to sanitize
        //   and render a "safe" version of the page, "mode=block" prevents any rendering
        //   that might still be exploitable.
        // 
        // - **Clear User Feedback**: Users see a clear error page indicating that
        //   potentially malicious content was blocked, rather than a partially broken page.
        // 
        // - **Prevents Bypass Attempts**: Some sophisticated XSS attacks try to exploit
        //   the browser's sanitization logic; blocking the page entirely prevents these
        //   bypass attempts.
        // 
        // ## Browser XSS Filter Mechanism:
        // 
        // Modern browsers include built-in XSS filters that:
        // 1. Analyze incoming requests and responses for potential XSS patterns
        // 2. Compare request parameters with response content to detect reflections
        // 3. Look for common XSS attack signatures and suspicious script patterns
        // 4. Take action based on the X-XSS-Protection header configuration
        // 
        // ## Limitations and Modern Context:
        // 
        // - **Browser Support**: Some modern browsers (like Chrome) have deprecated
        //   their XSS filters due to potential bypass techniques and false positives.
        // 
        // - **Not a Complete Solution**: X-XSS-Protection is a defense-in-depth measure
        //   and should not be relied upon as the primary XSS protection mechanism.
        // 
        // - **CSP is Preferred**: Content Security Policy (which we also implement)
        //   provides more robust and reliable XSS protection than browser XSS filters.
        // 
        // - **Legacy Support**: Including this header provides protection for older
        //   browsers and environments that still rely on XSS filters.
        // 
        // ## Why Include This Header for Our Static Server:
        // 
        // 1. **Defense in Depth**: Even though our content is static and trusted,
        //    this header provides an additional layer of protection.
        // 
        // 2. **Future Extensibility**: If the server is later extended to handle
        //    user input or dynamic content, this protection will already be in place.
        // 
        // 3. **Compliance**: Many security standards require this header to be present.
        // 
        // 4. **Legacy Browser Support**: Provides XSS protection for older browsers
        //    that may not fully support modern CSP directives.
        // 
        // 5. **Security Best Practice**: Including all relevant security headers
        //    demonstrates a comprehensive security posture.
        // 
        // ## Implementation Notes:
        // 
        // - The header name is case-insensitive but we use standard capitalization
        // - The "mode=block" parameter is case-sensitive and must be lowercase
        // - This header should be included on all HTML responses
        // - Some browsers may show a security warning when XSS is detected and blocked
        .header("x-xss-protection", "1; mode=block")  // Enable XSS filtering with blocking mode
        // Strict-Transport-Security Security Header (Task 23 - Requirements 3.4)
        // 
        // HTTP Strict Transport Security (HSTS) is a security mechanism that forces
        // browsers to use secure HTTPS connections when communicating with the server,
        // preventing various man-in-the-middle and protocol downgrade attacks.
        // 
        // ## What is HTTP Strict Transport Security (HSTS)?
        // 
        // HSTS is a web security policy mechanism that helps protect websites against
        // protocol downgrade attacks and cookie hijacking by forcing all communication
        // with the server to occur over secure HTTPS connections, even if the user
        // initially tries to access the site via HTTP.
        // 
        // ## Security Problems HSTS Solves:
        // 
        // 1. **Protocol Downgrade Attacks**: Attackers intercept initial HTTP requests
        //    and prevent the redirect to HTTPS, keeping the connection insecure.
        // 
        // 2. **Man-in-the-Middle (MITM) Attacks**: Attackers position themselves between
        //    the user and the server to intercept, modify, or steal data transmitted
        //    over insecure HTTP connections.
        // 
        // 3. **Cookie Hijacking**: Session cookies transmitted over HTTP can be
        //    intercepted by attackers on the same network (especially on public WiFi).
        // 
        // 4. **Mixed Content Issues**: Pages loaded over HTTPS that reference HTTP
        //    resources can be compromised by attackers who control those HTTP resources.
        // 
        // 5. **SSL Stripping Attacks**: Attackers remove HTTPS links from web pages,
        //    forcing users to connect over insecure HTTP instead of HTTPS.
        // 
        // ## How HSTS Works:
        // 
        // 1. **Initial HTTPS Connection**: User connects to the website over HTTPS
        //    (either directly or via HTTP redirect).
        // 
        // 2. **HSTS Header Received**: Server sends the Strict-Transport-Security header
        //    with the HTTPS response, instructing the browser to remember this policy.
        // 
        // 3. **Browser Policy Storage**: Browser stores the HSTS policy for the specified
        //    domain and duration (max-age period).
        // 
        // 4. **Automatic HTTPS Enforcement**: For the duration of the policy, the browser
        //    automatically converts all HTTP requests to the domain into HTTPS requests,
        //    even if the user types "http://" or clicks on HTTP links.
        // 
        // 5. **Certificate Validation**: Browser enforces strict certificate validation
        //    and will not allow users to bypass certificate errors for HSTS-enabled sites.
        // 
        // ## Our HSTS Policy Breakdown:
        // 
        // **max-age=31536000**: This specifies that the HSTS policy should remain in
        // effect for 31,536,000 seconds, which equals exactly one year (365 days × 24
        // hours × 60 minutes × 60 seconds). During this time, the browser will:
        // - Automatically redirect all HTTP requests to HTTPS
        // - Refuse to connect if there are certificate errors
        // - Not allow users to bypass certificate warnings
        // 
        // ## Why One Year is Appropriate:
        // 
        // - **Security vs Flexibility Balance**: Long enough to provide meaningful
        //   protection against attacks, but not so long that it becomes difficult
        //   to change if needed.
        // 
        // - **Industry Standard**: One year (31536000 seconds) is a common choice
        //   for HSTS max-age values in production applications.
        // 
        // - **Preload List Compatibility**: If we later want to submit our domain
        //   to the HSTS preload list, a minimum max-age of one year is required.
        // 
        // ## Optional HSTS Directives (Not Used in Our Implementation):
        // 
        // - **includeSubDomains**: Would apply HSTS policy to all subdomains as well.
        //   We don't include this because our Lambda Function URL is a single endpoint
        //   without subdomains we control.
        // 
        // - **preload**: Indicates that the domain owner consents to have their domain
        //   included in browsers' HSTS preload lists. This requires additional steps
        //   and is typically used for high-security applications.
        // 
        // ## Why HSTS is Important for Our Lambda Function:
        // 
        // 1. **AWS Lambda Function URLs Use HTTPS**: Lambda Function URLs are served
        //    over HTTPS by default, making HSTS enforcement meaningful and appropriate.
        // 
        // 2. **Prevents Downgrade Attacks**: Even though our content is static, HSTS
        //    prevents attackers from forcing users to connect over insecure HTTP.
        // 
        // 3. **Protects User Privacy**: Ensures that all communication with our server
        //    is encrypted, protecting user IP addresses and browsing patterns.
        // 
        // 4. **Future-Proof Security**: If the application is later extended with
        //    sensitive functionality, HSTS protection will already be in place.
        // 
        // 5. **Compliance Requirements**: Many security frameworks and compliance
        //    standards require HSTS for web applications.
        // 
        // ## HSTS Preload Lists:
        // 
        // Major browsers maintain HSTS preload lists - hardcoded lists of domains
        // that should always be accessed over HTTPS, even on the very first visit.
        // Domains can be submitted to these lists for maximum security, but this
        // requires careful consideration as removal can be difficult.
        // 
        // ## Implementation Notes:
        // 
        // - HSTS headers are only processed when received over HTTPS connections
        // - The max-age value is in seconds and must be a non-negative integer
        // - Browsers will ignore HSTS headers received over HTTP connections
        // - The policy persists across browser sessions and survives browser restarts
        // - Users cannot bypass HSTS policies (this is intentional for security)
        // 
        // ## HSTS and AWS Lambda Function URLs:
        // 
        // AWS Lambda Function URLs automatically provide HTTPS endpoints, making HSTS
        // a natural fit. The Function URL format is:
        // https://<url-id>.lambda-url.<region>.on.aws/
        // 
        // Since these are always HTTPS and we control the response headers, we can
        // effectively use HSTS to ensure users always connect securely.
        .header("strict-transport-security", "max-age=31536000")  // Enforce HTTPS for 1 year
        .body(HTML_CONTENT.into())  // Convert our static HTML string into response body
        .map_err(Box::new)?;  // Convert builder errors to Lambda Error type
    
    Ok(response)
}

/// Creates an error response with the specified status code and message
/// 
/// This function provides a consistent way to create error responses across
/// the application. All error responses use plain text content type and
/// include appropriate security headers.
/// 
/// ## Parameters:
/// - `status_code`: HTTP status code (400, 405, 413, 500, etc.)
/// - `message`: Error message to include in the response body
/// 
/// ## Security Considerations:
/// - Error messages should be generic to avoid information disclosure
/// - All error responses include the same security headers as success responses
/// - Content-Type is set to "text/plain" for error messages
pub fn create_error_response(status_code: u16, message: &str) -> Result<Response<Body>, Error> {
    let mut response_builder = Response::builder()
        .status(status_code)
        .header("content-type", "text/plain")  // Plain text for error messages
        .header("x-frame-options", "DENY")  // Prevent clickjacking attacks
        .header("x-content-type-options", "nosniff")  // Security header for all responses
        .header("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'")  // Restrict resource loading
        .header("x-xss-protection", "1; mode=block")  // Enable XSS filtering with blocking mode
        .header("strict-transport-security", "max-age=31536000");  // Enforce HTTPS for 1 year
    
    // Add Allow header for 405 Method Not Allowed responses
    if status_code == 405 {
        response_builder = response_builder.header("allow", "GET");
    }
    
    let response = response_builder
        .body(message.into())
        .map_err(Box::new)?;
    
    Ok(response)
}