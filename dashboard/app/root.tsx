import { Suspense } from "react";
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
import { cn } from "~/lib/utils";
import { Separator } from "~/components/ui/separator";
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

const navItems = [
  { to: "/", label: "Dashboard", end: true },
  { to: "/config", label: "Config" },
  { to: "/studio", label: "Rhai Studio" },
  { to: "/pinger", label: "Pinger" },
];

// Shown during the initial SPA hydration (blank page phase before React loads).
export function HydrateFallback() {
  return (
    <div className="flex items-center justify-center min-h-screen text-muted-foreground text-sm animate-pulse">
      Loading…
    </div>
  );
}

function RouteFallback() {
  return (
    <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm animate-pulse">
      Loading…
    </div>
  );
}

export default function App() {
  return (
    <div className="w-full px-6 py-6 flex flex-col min-h-screen">
      <header className="flex flex-col items-center gap-4 mb-8 pb-4 border-b border-border">
        <h1 className="text-xl font-semibold tracking-tight">
          <span className="text-primary">▶</span> event-generator
        </h1>
        <nav className="flex gap-1">
          {navItems.map(({ to, label, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end}
              className={({ isActive }) =>
                cn(
                  "h-8 px-3 flex items-center text-xs rounded-md border transition-colors cursor-pointer",
                  isActive
                    ? "border-primary text-foreground bg-primary/10"
                    : "border-transparent text-muted-foreground hover:text-foreground hover:bg-accent"
                )
              }
            >
              {label}
            </NavLink>
          ))}
        </nav>
      </header>

      <main className="flex-1 flex flex-col min-h-0">
        {/*
          Suspense here catches two things:
          1. clientLoader suspensions during route navigation
          2. React.lazy() components that haven't loaded yet
        */}
        <Suspense fallback={<RouteFallback />}>
          <Outlet />
        </Suspense>
      </main>

      <Separator className="mt-6" />
      <footer className="pt-3 text-center text-[0.7rem] text-muted-foreground">
        event-generator · Rust + Rhai · stats via WebSocket
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
      <h1 className="text-2xl font-bold text-destructive">{message}</h1>
      <p className="mt-2 text-muted-foreground">{details}</p>
      {stack && (
        <pre className="w-full p-4 overflow-x-auto mt-4 bg-card rounded-lg text-xs border border-border">
          <code>{stack}</code>
        </pre>
      )}
    </main>
  );
}
