import { useForm, type Resolver } from "react-hook-form";
import { z } from "zod";
import { v7 as uuidv7 } from "uuid";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";

const schema = z.object({
  name: z.string().min(1, "name is required").max(80),
  prefix: z.string().regex(/^[A-Z]{2,8}$/, "2-8 uppercase letters"),
});
type FormValues = z.infer<typeof schema>;

const resolver: Resolver<FormValues> = (values) => {
  const r = schema.safeParse(values);
  if (r.success) return { values: r.data, errors: {} };
  const errors: Record<string, { type: string; message: string }> = {};
  for (const issue of r.error.issues) {
    const key = issue.path[0];
    if (typeof key === "string" && !errors[key]) {
      errors[key] = { type: "validation", message: issue.message };
    }
  }
  return { values: {}, errors };
};

export interface NewProjectDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewProjectDialog({ open, onOpenChange }: NewProjectDialogProps) {
  const apply = useApply();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({ resolver, defaultValues: { name: "", prefix: "" } });

  if (!open) return null;

  const onSubmit = handleSubmit((values) => {
    apply.mutate(ops.createProject({ id: uuidv7(), name: values.name, prefix: values.prefix }));
    reset();
    onOpenChange(false);
  });

  const cancel = () => {
    reset();
    onOpenChange(false);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      role="dialog"
      aria-modal="true"
      aria-label="New project"
    >
      <div className="w-full max-w-sm rounded-lg bg-white p-5 shadow-xl dark:bg-neutral-900">
        <h2 className="mb-4 text-base font-semibold">New project</h2>
        <form onSubmit={onSubmit} className="flex flex-col gap-4">
          <label className="flex flex-col gap-1 text-sm">
            <span>Name</span>
            <input
              {...register("name")}
              autoFocus
              className="rounded-md border border-black/10 px-3 py-1.5 text-sm dark:border-white/10 dark:bg-neutral-800"
            />
            {errors.name?.message && (
              <span className="text-xs text-red-500">{errors.name.message}</span>
            )}
          </label>
          <label className="flex flex-col gap-1 text-sm">
            <span>Prefix</span>
            <input
              {...register("prefix")}
              className="rounded-md border border-black/10 px-3 py-1.5 font-mono text-sm uppercase dark:border-white/10 dark:bg-neutral-800"
            />
            {errors.prefix?.message && (
              <span className="text-xs text-red-500">{errors.prefix.message}</span>
            )}
          </label>
          <div className="mt-2 flex justify-end gap-2">
            <button
              type="button"
              onClick={cancel}
              className="rounded-md px-3 py-1.5 text-sm text-neutral-600 hover:bg-black/5 dark:text-neutral-300 dark:hover:bg-white/10"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={apply.isPending}
              className="rounded-md bg-neutral-900 px-3 py-1.5 text-sm text-white disabled:opacity-50 dark:bg-white dark:text-neutral-900"
            >
              {apply.isPending ? "Creating…" : "Create"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
