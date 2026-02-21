import {
  isRouteErrorResponse,
  Links,
  Meta,
  NavLink,
  Outlet,
  Scripts,
  ScrollRestoration,
} from "react-router";

import type { Route } from "./+types/root";
import "./app.css";

export const links: Route.LinksFunction = () => [];

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Event Generator</title>
        <Meta />
        <Links />
      </head>
      <body>
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App() {
  return (
    <div className="max-w-[1200px] mx-auto px-6 py-6 flex flex-col min-h-screen">
      <header className="flex items-center justify-between mb-8 pb-4 border-b border-border">
        <h1 className="text-xl font-semibold tracking-tight">
          <span className="text-accent">&#9654;</span> event-generator
        </h1>
        <nav className="flex gap-2">
          <NavLink
            to="/"
            end
            className={({ isActive }) =>
              `h-[34px] px-3 flex items-center text-xs border rounded-lg cursor-pointer transition-colors ${
                isActive
                  ? "border-accent text-text"
                  : "border-border text-text-dim bg-surface hover:text-text"
              }`
            }
          >
            Dashboard
          </NavLink>
          <NavLink
            to="/config"
            className={({ isActive }) =>
              `h-[34px] px-3 flex items-center text-xs border rounded-lg cursor-pointer transition-colors ${
                isActive
                  ? "border-accent text-text"
                  : "border-border text-text-dim bg-surface hover:text-text"
              }`
            }
          >
            Config
          </NavLink>
          <NavLink
            to="/studio"
            className={({ isActive }) =>
              `h-[34px] px-3 flex items-center text-xs border rounded-lg cursor-pointer transition-colors ${
                isActive
                  ? "border-accent text-text"
                  : "border-border text-text-dim bg-surface hover:text-text"
              }`
            }
          >
            Rhai Studio
          </NavLink>
        </nav>
      </header>
      <main className="flex-1 flex flex-col min-h-0">
        <Outlet />
      </main>
      <footer className="pt-4 mt-6 border-t border-border text-center text-[0.7rem] text-text-dim">
        event-generator &middot; Rust + Rhai &middot; stats via WebSocket
      </footer>
    </div>
  );
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  let message = "Oops!";
  let details = "An unexpected error occurred.";
  let stack: string | undefined;

  if (isRouteErrorResponse(error)) {
    message = error.status === 404 ? "404" : "Error";
    details =
      error.status === 404
        ? "The requested page could not be found."
        : error.statusText || details;
  } else if (import.meta.env.DEV && error && error instanceof Error) {
    details = error.message;
    stack = error.stack;
  }

  return (
    <main className="pt-16 p-4 container mx-auto">
      <h1 className="text-2xl font-bold text-red">{message}</h1>
      <p className="mt-2 text-text-dim">{details}</p>
      {stack && (
        <pre className="w-full p-4 overflow-x-auto mt-4 bg-surface rounded-lg text-xs">
          <code>{stack}</code>
        </pre>
      )}
    </main>
  );
}
