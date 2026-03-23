// Dashboard stats endpoint — Cloudflare Pages Function
// Queries D1 for aggregated analytics. Requires DASHBOARD_TOKEN env var.

const PERIODS = {
    day: 1,
    week: 7,
    month: 30,
};

const CACHE_MAX_AGE = 43200; // 12 hours in seconds

export async function onRequestGet(context) {
    const url = new URL(context.request.url);
    const token = url.searchParams.get('token');

    if (!context.env.DASHBOARD_TOKEN || token !== context.env.DASHBOARD_TOKEN) {
        return new Response('unauthorized', { status: 401 });
    }

    const periodKey = url.searchParams.get('period') || 'week';
    const days = PERIODS[periodKey];
    if (!days) {
        return new Response('invalid period: use day, week, or month', { status: 400 });
    }

    const bust = url.searchParams.get('bust') === '1';
    const since = new Date(Date.now() - days * 86400000).toISOString();

    try {
        const [pageviews, topPage, avgTime, topAudioFile, topAudioPage] =
            await Promise.all([
                totalPageviews(context.env.ADB, since),
                mostActivePage(context.env.ADB, since),
                avgTimeOnPage(context.env.ADB, since),
                topAudioFileByCompletions(context.env.ADB, since),
                topAudioPageByCompletions(context.env.ADB, since),
            ]);

        const body = JSON.stringify({
            period: periodKey,
            since: since,
            total_pageviews: pageviews,
            most_active_page: topPage,
            avg_time_on_page: avgTime,
            top_audio_file: topAudioFile,
            top_audio_page: topAudioPage,
        });

        const headers = {
            'Content-Type': 'application/json',
        };
        if (!bust) {
            headers['Cache-Control'] = 'public, max-age=' + CACHE_MAX_AGE;
        } else {
            headers['Cache-Control'] = 'no-store';
        }

        return new Response(body, { status: 200, headers: headers });
    } catch (err) {
        return new Response('db error', { status: 500 });
    }
}

async function totalPageviews(db, since) {
    const row = await db
        .prepare("SELECT COUNT(*) AS cnt FROM events WHERE event = 'pageview' AND ts >= ?")
        .bind(since)
        .first();
    return row ? row.cnt : 0;
}

async function mostActivePage(db, since) {
    const row = await db
        .prepare(
            "SELECT page, COUNT(*) AS cnt FROM events WHERE event = 'pageview' AND ts >= ? GROUP BY page ORDER BY cnt DESC LIMIT 1"
        )
        .bind(since)
        .first();
    return row ? { page: row.page, views: row.cnt } : null;
}

async function avgTimeOnPage(db, since) {
    const row = await db
        .prepare(
            "SELECT AVG(CAST(value AS REAL)) AS avg_seconds FROM events WHERE event = 'time_on_page' AND ts >= ? AND CAST(value AS REAL) > 0"
        )
        .bind(since)
        .first();
    return row && row.avg_seconds != null ? Math.round(row.avg_seconds) : null;
}

async function topAudioFileByCompletions(db, since) {
    const row = await db
        .prepare(
            "SELECT value, COUNT(*) AS cnt FROM events WHERE event = 'audio_complete' AND ts >= ? AND value IS NOT NULL GROUP BY value ORDER BY cnt DESC LIMIT 1"
        )
        .bind(since)
        .first();
    return row ? { file: row.value, completions: row.cnt } : null;
}

async function topAudioPageByCompletions(db, since) {
    const row = await db
        .prepare(
            "SELECT page, COUNT(*) AS cnt FROM events WHERE event = 'audio_complete' AND ts >= ? GROUP BY page ORDER BY cnt DESC LIMIT 1"
        )
        .bind(since)
        .first();
    return row ? { page: row.page, completions: row.cnt } : null;
}
