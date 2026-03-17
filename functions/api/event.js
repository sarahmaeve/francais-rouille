// Analytics event ingestion — Cloudflare Pages Function
// Receives JSON events from shared/analytics.js and writes to D1.

const VALID_EVENTS = new Set([
    'pageview',
    'time_on_page',
    'audio_complete',
    'audio_play',
    'quiz_complete',
]);

// Maximum lengths to prevent abuse.
const MAX_SESSION = 64;
const MAX_PAGE = 256;
const MAX_VALUE = 512;

export async function onRequestPost(context) {
    let body;
    try {
        body = await context.request.json();
    } catch {
        return new Response('invalid json', { status: 400 });
    }

    const { session, event, page, value } = body;

    if (!session || !event || !page) {
        return new Response('missing fields', { status: 400 });
    }

    if (!VALID_EVENTS.has(event)) {
        return new Response('unknown event', { status: 400 });
    }

    if (session.length > MAX_SESSION || page.length > MAX_PAGE) {
        return new Response('field too long', { status: 400 });
    }

    const safeValue = value != null ? String(value).slice(0, MAX_VALUE) : null;
    const ts = new Date().toISOString();

    try {
        await context.env.ADB.prepare(
            'INSERT INTO events (ts, session, event, page, value) VALUES (?, ?, ?, ?, ?)'
        ).bind(ts, session, event, page, safeValue).run();
    } catch (err) {
        return new Response('db error', { status: 500 });
    }

    return new Response('ok', { status: 202 });
}

// Reject non-POST methods.
export async function onRequestGet() {
    return new Response('method not allowed', { status: 405 });
}
