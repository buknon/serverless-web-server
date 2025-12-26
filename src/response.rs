// HTTP response creation and formatting functions
// This module handles building proper HTTP responses with security headers

use lambda_http::{Error, Response, Body};
use log;
use chrono;
use std::env;

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

/// Generates or extracts a request ID for error correlation and logging
/// 
/// This function implements request ID generation for error correlation as required by
/// Requirements 5.4: "Include request ID for error correlation"
/// 
/// ## Request ID Sources (in order of preference):
/// 
/// 1. **AWS Lambda Request ID**: AWS Lambda automatically provides a unique request ID
///    for each invocation via the `AWS_LAMBDA_LOG_STREAM_NAME` and `_X_AMZN_TRACE_ID`
///    environment variables. This is the preferred source as it integrates with AWS
///    CloudWatch and X-Ray tracing.
/// 
/// 2. **Generated UUID**: If AWS Lambda environment variables are not available
///    (e.g., during local testing), we generate a random UUID for request tracking.
/// 
/// ## Request ID Format:
/// 
/// - **AWS Lambda**: Uses the format provided by AWS (typically UUID-like)
/// - **Generated**: Uses a simple timestamp-based format for local development
/// 
/// ## Use Cases:
/// 
/// - **Error Correlation**: Link error logs with specific requests for debugging
/// - **Distributed Tracing**: Integrate with AWS X-Ray and CloudWatch for request tracking
/// - **Security Monitoring**: Track security violations across multiple log entries
/// - **Performance Analysis**: Correlate request processing times with specific requests
/// - **Incident Response**: Quickly identify all log entries related to a problematic request
/// 
/// ## Security Considerations:
/// 
/// - Request IDs are safe to include in user-facing error messages
/// - They don't reveal sensitive information about the system or request
/// - They provide a way for users to reference specific errors when reporting issues
/// - They enable security teams to correlate attacks across multiple requests
/// 
/// ## Implementation Notes:
/// 
/// - This function is called once per request to ensure consistent request ID usage
/// - The request ID is included in both detailed error logs and generic user messages
/// - AWS Lambda request IDs are automatically correlated with CloudWatch logs
/// - Generated request IDs are useful for local development and testing
/// 
/// ## Return Value:
/// 
/// Returns a string containing a unique request identifier that can be safely
/// included in both logs and user-facing error messages.
fn generate_request_id() -> String {
    // Try to get AWS Lambda request ID from environment variables
    // AWS Lambda provides several environment variables that can be used for request correlation:
    // - AWS_LAMBDA_LOG_STREAM_NAME: Contains the log stream name which includes request info
    // - _X_AMZN_TRACE_ID: Contains AWS X-Ray trace ID for distributed tracing
    // - AWS_LAMBDA_REQUEST_ID: Direct request ID (if available in newer Lambda runtimes)
    
    // First, try to get the X-Ray trace ID which is most useful for correlation
    if let Ok(trace_id) = env::var("_X_AMZN_TRACE_ID") {
        // Extract just the trace ID portion (before any additional metadata)
        // X-Ray trace IDs have the format: Root=1-5e1b4151-5ac6c58f5b5dcc1e1e0a7e1c;Parent=...;Sampled=...
        if let Some(root_part) = trace_id.split(';').next() {
            if let Some(trace_part) = root_part.strip_prefix("Root=") {
                return format!("trace-{}", trace_part);
            }
        }
        // If parsing fails, use the full trace ID (truncated for readability)
        return format!("trace-{}", &trace_id[..std::cmp::min(trace_id.len(), 32)]);
    }
    
    // Try to get AWS Lambda request ID directly (newer runtimes)
    if let Ok(request_id) = env::var("AWS_LAMBDA_REQUEST_ID") {
        return format!("lambda-{}", request_id);
    }
    
    // Try to extract request ID from log stream name
    // Log stream names typically contain request-specific information
    if let Ok(log_stream) = env::var("AWS_LAMBDA_LOG_STREAM_NAME") {
        // Extract the last part of the log stream name which often contains request info
        if let Some(last_part) = log_stream.split('/').last() {
            return format!("stream-{}", last_part);
        }
    }
    
    // Fallback: Generate a timestamp-based request ID for local development
    // This ensures we always have a request ID even when running outside AWS Lambda
    let timestamp = chrono::Utc::now();
    let timestamp_str = timestamp.format("%Y%m%d-%H%M%S-%3f").to_string();
    
    // Add a random component to ensure uniqueness even for concurrent requests
    let random_suffix = timestamp.timestamp_nanos_opt().unwrap_or(0) % 10000;
    
    format!("local-{}-{:04}", timestamp_str, random_suffix)
}

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

/// Application error types that can occur during request processing
/// 
/// This enum represents all possible error conditions that can occur in our
/// Lambda function. Each error type maps to an appropriate HTTP status code
/// and generic user message to prevent information disclosure.
/// 
/// ## Security Principles:
/// 
/// 1. **Generic User Messages**: All error messages shown to users are generic
///    and don't reveal internal implementation details or sensitive information.
/// 
/// 2. **Detailed Logging**: Full error details are logged internally for
///    debugging and security monitoring, but never exposed to users.
/// 
/// 3. **Consistent HTTP Status Codes**: Each error type maps to the most
///    appropriate HTTP status code according to RFC standards.
/// 
/// 4. **Information Disclosure Prevention**: Error responses don't reveal
///    system internals, file paths, database schemas, or other sensitive data.
#[derive(Debug, Clone)]
pub enum ApplicationError {
    /// Security-related errors (malicious requests, validation failures)
    /// 
    /// These errors occur when security validation fails, such as:
    /// - Invalid HTTP methods
    /// - Malicious request paths
    /// - Oversized requests
    /// - Suspicious headers or content
    /// 
    /// **User Message**: Generic security error message
    /// **HTTP Status**: Varies based on specific security violation
    /// **Logging**: Full security violation details for monitoring
    Security {
        /// The underlying security error with full details
        security_error: crate::security::SecurityError,
        /// Additional context about when/where the error occurred
        context: String,
    },

    /// Internal server errors (unexpected failures, system errors)
    /// 
    /// These errors occur when something goes wrong internally that's not
    /// the user's fault, such as:
    /// - Memory allocation failures
    /// - Unexpected panics or crashes
    /// - AWS Lambda runtime errors
    /// - Configuration errors
    /// 
    /// **User Message**: Generic internal server error message
    /// **HTTP Status**: 500 Internal Server Error
    /// **Logging**: Full error details and stack traces for debugging
    InternalError {
        /// Description of what went wrong internally
        details: String,
        /// Optional underlying error cause
        cause: Option<String>,
    },

    /// Request processing errors (malformed requests, invalid data)
    /// 
    /// These errors occur when the request is malformed or contains invalid
    /// data that prevents normal processing, such as:
    /// - Invalid request format
    /// - Missing required headers
    /// - Corrupted request data
    /// - Unsupported request features
    /// 
    /// **User Message**: Generic bad request message
    /// **HTTP Status**: 400 Bad Request
    /// **Logging**: Request details for debugging (sanitized)
    RequestError {
        /// Description of what's wrong with the request
        details: String,
        /// The problematic request component (headers, body, path, etc.)
        component: String,
    },

    /// Service unavailable errors (temporary failures, rate limiting)
    /// 
    /// These errors occur when the service is temporarily unable to process
    /// requests due to:
    /// - System overload
    /// - Temporary resource exhaustion
    /// - Maintenance mode
    /// - Rate limiting
    /// 
    /// **User Message**: Generic service unavailable message
    /// **HTTP Status**: 503 Service Unavailable
    /// **Logging**: Service status and resource usage details
    ServiceUnavailable {
        /// Reason why the service is unavailable
        reason: String,
        /// Estimated time until service recovery (if known)
        retry_after: Option<u32>,
    },
}

impl ApplicationError {
    /// Converts an ApplicationError to the appropriate HTTP status code
    /// 
    /// This method maps each application error type to its corresponding HTTP
    /// status code according to HTTP standards and security best practices.
    /// 
    /// ## Status Code Mapping:
    /// 
    /// - **Security Errors**: Use the specific status code from the security error
    ///   - 400 Bad Request: For malformed or malicious requests
    ///   - 405 Method Not Allowed: For unsupported HTTP methods
    ///   - 413 Request Entity Too Large: For oversized requests
    /// 
    /// - **Internal Errors**: 500 Internal Server Error
    ///   - Used for unexpected server-side failures
    ///   - Indicates the problem is not the client's fault
    /// 
    /// - **Request Errors**: 400 Bad Request
    ///   - Used for malformed or invalid client requests
    ///   - Indicates the client needs to fix their request
    /// 
    /// - **Service Unavailable**: 503 Service Unavailable
    ///   - Used for temporary service outages or overload
    ///   - Indicates the client should retry later
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            ApplicationError::Security { security_error, .. } => {
                security_error.to_http_status_code()
            }
            ApplicationError::InternalError { .. } => 500, // Internal Server Error
            ApplicationError::RequestError { .. } => 400, // Bad Request
            ApplicationError::ServiceUnavailable { .. } => 503, // Service Unavailable
        }
    }

    /// Returns a generic error message safe for displaying to users
    /// 
    /// This method provides user-facing error messages that are generic enough
    /// to avoid information disclosure while still being helpful to legitimate users.
    /// 
    /// ## Security Principles:
    /// 
    /// 1. **No Information Disclosure**: Messages don't reveal internal details,
    ///    system architecture, file paths, or implementation specifics.
    /// 
    /// 2. **Generic but Helpful**: Provide enough information for legitimate users
    ///    to understand what went wrong and potentially fix their request.
    /// 
    /// 3. **Consistent Format**: All messages follow the same professional tone
    ///    and structure for a consistent user experience.
    /// 
    /// 4. **No Attack Vectors**: Messages don't contain any content that could
    ///    be used for further attacks or reconnaissance.
    /// 
    /// ## Message Design Philosophy:
    /// 
    /// - Brief and clear explanations appropriate for end users
    /// - Professional tone suitable for production applications
    /// - No technical jargon that might confuse non-technical users
    /// - Consistent formatting and punctuation across all error types
    /// - Actionable guidance when appropriate (e.g., "try again later")
    pub fn to_generic_user_message(&self) -> String {
        match self {
            ApplicationError::Security { security_error, .. } => {
                // Use the security error's user-safe message
                // These are already designed to be generic and safe
                security_error.to_user_message()
            }
            ApplicationError::InternalError { .. } => {
                // Generic message for internal server errors
                // Don't reveal any details about what went wrong internally
                "Internal Server Error. Please try again later.".to_string()
            }
            ApplicationError::RequestError { .. } => {
                // Generic message for request errors
                // Don't reveal specific details about what's wrong with the request
                "Bad Request. Please check your request and try again.".to_string()
            }
            ApplicationError::ServiceUnavailable { retry_after, .. } => {
                // Generic message for service unavailability
                // Optionally include retry guidance if available
                match retry_after {
                    Some(seconds) => {
                        format!("Service Temporarily Unavailable. Please try again in {} seconds.", seconds)
                    }
                    None => {
                        "Service Temporarily Unavailable. Please try again later.".to_string()
                    }
                }
            }
        }
    }

    /// Returns detailed error information for logging and debugging
    /// 
    /// This method provides comprehensive error details that should only be
    /// used for internal logging, monitoring, and debugging. These details
    /// should never be exposed to end users as they may contain sensitive
    /// information about the system's internal workings.
    /// 
    /// ## Security Considerations:
    /// 
    /// - Contains sensitive information about system internals
    /// - Includes full error details for forensic analysis
    /// - Should only be used for internal logging and monitoring
    /// - Helps developers and security teams understand and fix issues
    /// - May contain stack traces, file paths, and system information
    /// 
    /// ## Use Cases:
    /// 
    /// - Application debugging and troubleshooting
    /// - Security incident response and analysis
    /// - Performance monitoring and optimization
    /// - Compliance logging and audit trails
    /// - Error tracking and alerting systems
    pub fn to_detailed_message(&self) -> String {
        match self {
            ApplicationError::Security { security_error, context } => {
                format!(
                    "Security Error in {}: {}",
                    context,
                    security_error.to_detailed_message()
                )
            }
            ApplicationError::InternalError { details, cause } => {
                match cause {
                    Some(cause_info) => {
                        format!("Internal Error: {} (Cause: {})", details, cause_info)
                    }
                    None => {
                        format!("Internal Error: {}", details)
                    }
                }
            }
            ApplicationError::RequestError { details, component } => {
                format!("Request Error in {}: {}", component, details)
            }
            ApplicationError::ServiceUnavailable { reason, retry_after } => {
                match retry_after {
                    Some(seconds) => {
                        format!("Service Unavailable: {} (Retry after {} seconds)", reason, seconds)
                    }
                    None => {
                        format!("Service Unavailable: {}", reason)
                    }
                }
            }
        }
    }

    /// Returns the error type name for categorization and alerting
    /// 
    /// This method provides a simple string identifier for each error type
    /// that can be used for log filtering, alerting, and error categorization.
    /// 
    /// ## Use Cases:
    /// 
    /// - Log filtering and searching (e.g., filter all "Security" errors)
    /// - Error monitoring and alerting systems
    /// - Error rate tracking by category
    /// - Automated incident response based on error type
    /// - Performance monitoring and SLA tracking
    /// 
    /// ## Return Values:
    /// 
    /// - "Security": For security-related errors (malicious requests, validation failures)
    /// - "Internal": For internal server errors (unexpected failures, system errors)
    /// - "Request": For request processing errors (malformed requests, invalid data)
    /// - "ServiceUnavailable": For service unavailable errors (temporary failures, rate limiting)
    pub fn error_type_name(&self) -> &'static str {
        match self {
            ApplicationError::Security { .. } => "Security",
            ApplicationError::InternalError { .. } => "Internal",
            ApplicationError::RequestError { .. } => "Request",
            ApplicationError::ServiceUnavailable { .. } => "ServiceUnavailable",
        }
    }
}

/// Creates a generic error response that prevents information disclosure
/// 
/// This function is the primary interface for creating error responses in our
/// application. It ensures that all error responses follow security best practices
/// by providing generic user messages while logging detailed error information.
/// 
/// ## Security Features:
/// 
/// 1. **Generic User Messages**: All error messages shown to users are generic
///    and don't reveal internal system details, implementation specifics, or
///    sensitive information that could be used by attackers.
/// 
/// 2. **Comprehensive Logging**: Full error details are logged internally for
///    debugging, security monitoring, and incident response, but never exposed
///    to end users.
/// 
/// 3. **Consistent Security Headers**: All error responses include the same
///    security headers as successful responses to maintain consistent security
///    posture across all endpoints.
/// 
/// 4. **Appropriate HTTP Status Codes**: Each error type maps to the most
///    appropriate HTTP status code according to RFC standards.
/// 
/// ## Information Disclosure Prevention:
/// 
/// This function prevents several types of information disclosure:
/// 
/// - **System Architecture**: No details about internal system structure
/// - **File Paths**: No server file system paths or directory structures
/// - **Database Schemas**: No database table names, column names, or queries
/// - **Stack Traces**: No internal code execution paths or function names
/// - **Configuration Details**: No server configuration or environment info
/// - **Third-party Services**: No details about external dependencies
/// - **Error Causes**: No specific reasons why operations failed internally
/// 
/// ## Error Response Format:
/// 
/// All error responses use plain text content type and include:
/// - Appropriate HTTP status code
/// - Generic, user-friendly error message
/// - Complete set of security headers
/// - Allow header for 405 Method Not Allowed responses
/// 
/// ## Parameters:
/// - `error`: The ApplicationError containing full error details
/// 
/// ## Return Value:
/// - `Ok(Response<Body>)`: Successfully created error response
/// - `Err(Error)`: Failed to create response (rare, indicates system issue)
/// 
/// ## Usage Examples:
/// 
/// ```text
/// // Security error
/// let security_err = ApplicationError::Security {
///     security_error: SecurityError::InvalidMethod { 
///         method: "POST".to_string(), 
///         path: "/".to_string() 
///     },
///     context: "request validation".to_string(),
/// };
/// let response = create_generic_error_response(security_err)?;
/// 
/// // Internal error
/// let internal_err = ApplicationError::InternalError {
///     details: "Failed to allocate memory for response".to_string(),
///     cause: Some("Out of memory".to_string()),
/// };
/// let response = create_generic_error_response(internal_err)?;
/// ```
pub fn create_generic_error_response(error: ApplicationError) -> Result<Response<Body>, Error> {
    // Generate a unique request ID for error correlation (Task 30 - Requirements 5.4)
    // This enables correlation between user-facing error messages and detailed internal logs
    let request_id = generate_request_id();
    
    // Log the detailed error information for internal monitoring and debugging (Task 30 - Requirements 5.4)
    // This provides full context for developers and security teams while
    // keeping sensitive details away from end users
    // 
    // Enhanced logging format includes:
    // - Timestamp: ISO 8601 format for consistent time representation
    // - Request ID: Unique identifier for correlating this error with user reports
    // - HTTP Status Code: The status code that will be returned to the user
    // - Detailed Error: Full error details including sensitive information for debugging
    // - Error Type: The specific type of error for categorization and alerting
    log::error!(
        "[{}] [ERROR] [REQUEST_ID:{}] Returning generic error response: status={} error_type=\"{}\" detailed_error=\"{}\"",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        request_id,
        error.to_http_status_code(),
        error.error_type_name(),
        error.to_detailed_message()
    );
    
    // Additional structured logging for security monitoring and incident response
    // This separate log entry makes it easier to filter and alert on specific error types
    match &error {
        ApplicationError::Security { security_error, context } => {
            log::warn!(
                "[{}] [SECURITY_VIOLATION] [REQUEST_ID:{}] Security error in {}: {} (status={})",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                request_id,
                context,
                security_error.to_detailed_message(),
                security_error.to_http_status_code()
            );
        }
        ApplicationError::InternalError { details, cause } => {
            log::error!(
                "[{}] [INTERNAL_ERROR] [REQUEST_ID:{}] Internal system error: {} (cause: {})",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                request_id,
                details,
                cause.as_deref().unwrap_or("unknown")
            );
        }
        ApplicationError::RequestError { details, component } => {
            log::warn!(
                "[{}] [REQUEST_ERROR] [REQUEST_ID:{}] Invalid request in {}: {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                request_id,
                component,
                details
            );
        }
        ApplicationError::ServiceUnavailable { reason, retry_after } => {
            log::warn!(
                "[{}] [SERVICE_UNAVAILABLE] [REQUEST_ID:{}] Service unavailable: {} (retry_after: {})",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                request_id,
                reason,
                retry_after.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string())
            );
        }
    }
    
    // Get the appropriate HTTP status code for this error type
    let status_code = error.to_http_status_code();
    
    // Get the generic, user-safe error message with request ID for correlation
    // This message is designed to be helpful to legitimate users while
    // not revealing any sensitive information to potential attackers
    // The request ID allows users to reference specific errors when reporting issues
    let user_message = format!("{} (Request ID: {})", error.to_generic_user_message(), request_id);
    
    // Build the error response with consistent security headers
    let mut response_builder = Response::builder()
        .status(status_code)
        .header("content-type", "text/plain")  // Plain text for error messages
        // Include all security headers to maintain consistent security posture
        .header("x-frame-options", "DENY")  // Prevent clickjacking attacks
        .header("x-content-type-options", "nosniff")  // Prevent MIME type sniffing
        .header("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'")  // Restrict resource loading
        .header("x-xss-protection", "1; mode=block")  // Enable XSS filtering with blocking mode
        .header("strict-transport-security", "max-age=31536000");  // Enforce HTTPS for 1 year
    
    // Add Allow header for 405 Method Not Allowed responses
    // This tells the client which HTTP methods are supported
    if status_code == 405 {
        response_builder = response_builder.header("allow", "GET");
    }
    
    // Add Retry-After header for 503 Service Unavailable responses
    // This tells the client when they should try again
    if let ApplicationError::ServiceUnavailable { retry_after: Some(seconds), .. } = &error {
        response_builder = response_builder.header("retry-after", seconds.to_string());
    }
    
    // Build the final response with the generic user message including request ID
    let response = response_builder
        .body(user_message.into())
        .map_err(Box::new)?;
    
    Ok(response)
}

/// Creates an error response with the specified status code and message
/// 
/// This function provides a consistent way to create error responses across
/// the application. All error responses use plain text content type and
/// include appropriate security headers.
/// 
/// ## Deprecation Notice:
/// 
/// This function is maintained for backward compatibility, but new code should
/// use `create_generic_error_response()` with `ApplicationError` for better
/// security and consistency.
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