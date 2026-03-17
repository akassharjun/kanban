import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen w-screen items-center justify-center bg-zinc-950 text-zinc-100">
          <div className="max-w-md rounded-lg border border-zinc-800 bg-zinc-900 p-8 text-center">
            <h1 className="text-lg font-semibold text-zinc-100">
              Something went wrong
            </h1>
            <p className="mt-2 text-sm text-zinc-400">
              {this.state.error?.message || "An unexpected error occurred."}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="mt-6 rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
            >
              Reload
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
