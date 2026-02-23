import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";

export interface HeaderPair {
  key: string;
  value: string;
}

export function KeyValueList({
  pairs,
  onChange,
  keyPlaceholder,
  valuePlaceholder,
}: {
  pairs: HeaderPair[];
  onChange: (pairs: HeaderPair[]) => void;
  keyPlaceholder: string;
  valuePlaceholder: string;
}) {
  const update = (index: number, field: keyof HeaderPair, value: string) => {
    const next = pairs.map((p, i) => (i === index ? { ...p, [field]: value } : p));
    onChange(next);
  };

  const remove = (index: number) => onChange(pairs.filter((_, i) => i !== index));

  return (
    <div className="flex flex-col gap-2">
      {pairs.map((pair, i) => (
        <div key={i} className="flex gap-2 items-center">
          <Input
            value={pair.key}
            onChange={(e) => update(i, "key", e.target.value)}
            placeholder={keyPlaceholder}
            className="text-xs"
          />
          <Input
            value={pair.value}
            onChange={(e) => update(i, "value", e.target.value)}
            placeholder={valuePlaceholder}
            className="text-xs flex-[2]"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="shrink-0 h-8 w-8 text-muted-foreground hover:text-destructive"
            onClick={() => remove(i)}
          >
            ×
          </Button>
        </div>
      ))}
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="w-fit text-xs"
        onClick={() => onChange([...pairs, { key: "", value: "" }])}
      >
        + Add
      </Button>
    </div>
  );
}
