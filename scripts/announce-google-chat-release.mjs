#!/usr/bin/env node

import fs from "node:fs";

const MAX_BODY_CHARS = 900;

function requiredEnv(name) {
  const value = process.env[name]?.trim();
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function optionalEnv(name, fallback = "") {
  return process.env[name]?.trim() || fallback;
}

function releaseBody() {
  const direct = optionalEnv("RELEASE_BODY");
  if (direct) {
    return direct;
  }

  const path = optionalEnv("RELEASE_BODY_PATH");
  if (!path) {
    return "";
  }

  return fs.readFileSync(path, "utf8");
}

function stripMarkdown(markdown) {
  return markdown
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => !line.startsWith("## "))
    .filter((line) => !line.startsWith("```"))
    .map((line) =>
      line
        .replace(/^[-*]\s+/, "• ")
        .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
        .replace(/`([^`]+)`/g, "$1")
        .replace(/\*\*([^*]+)\*\*/g, "$1")
        .replace(/\*([^*]+)\*/g, "$1"),
    )
    .join("\n");
}

function excerpt(markdown) {
  const text = stripMarkdown(markdown);
  if (!text) {
    return "See the release notes for details.";
  }
  if (text.length <= MAX_BODY_CHARS) {
    return text;
  }
  return `${text.slice(0, MAX_BODY_CHARS - 1).trimEnd()}…`;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function webhookUrl(rawWebhook) {
  let url;
  try {
    url = new URL(rawWebhook);
  } catch {
    throw new Error("GCHAT_RELEASE_WEBHOOK must be a valid URL");
  }

  if (url.protocol !== "https:" || url.hostname !== "chat.googleapis.com") {
    throw new Error("GCHAT_RELEASE_WEBHOOK must be a Google Chat HTTPS webhook");
  }

  if (!url.searchParams.has("messageReplyOption")) {
    url.searchParams.set("messageReplyOption", "REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD");
  }

  return url;
}

function buildPayload({ title, subtitle, body, releaseUrl, threadKey }) {
  const escapedBody = escapeHtml(body).replaceAll("\n", "<br>");

  return {
    text: `${title}: ${subtitle}`,
    thread: {
      threadKey,
    },
    cardsV2: [
      {
        cardId: "kay-release-announcement",
        card: {
          header: {
            title,
            subtitle,
          },
          sections: [
            {
              widgets: [
                {
                  textParagraph: {
                    text: escapedBody,
                  },
                },
                {
                  buttonList: {
                    buttons: [
                      {
                        text: "View release",
                        onClick: {
                          openLink: {
                            url: releaseUrl,
                          },
                        },
                      },
                    ],
                  },
                },
              ],
            },
          ],
        },
      },
    ],
  };
}

async function main() {
  const rawWebhook = requiredEnv("GCHAT_RELEASE_WEBHOOK");
  const tag = requiredEnv("RELEASE_TAG");
  const releaseUrl = requiredEnv("RELEASE_URL");
  const releaseTitle = optionalEnv("RELEASE_TITLE", `Release ${tag}`);
  const title = optionalEnv("GCHAT_RELEASE_TITLE", "Kay Updates");
  const threadKey = optionalEnv("GCHAT_RELEASE_THREAD_KEY", "kay-updates");
  const dryRun = optionalEnv("GCHAT_RELEASE_DRY_RUN") === "1";

  const payload = buildPayload({
    title,
    subtitle: releaseTitle,
    body: excerpt(releaseBody()),
    releaseUrl,
    threadKey,
  });

  if (dryRun) {
    console.log(JSON.stringify(payload, null, 2));
    return;
  }

  const response = await fetch(webhookUrl(rawWebhook), {
    method: "POST",
    headers: {
      "Content-Type": "application/json; charset=utf-8",
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    const responseBody = (await response.text()).slice(0, 1000);
    throw new Error(
      `Google Chat announcement failed with HTTP ${response.status}: ${responseBody}`,
    );
  }

  console.log(`Google Chat release announcement sent for ${releaseUrl}`);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
