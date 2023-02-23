def dump_to_hex(obj)
  e = Marshal.dump(obj)
  e = e.bytes.map { |b| "0x" + b.to_s(16).rjust(2, "0") }.join(", ")

  puts "&[#{e}];"
end

dump_to_hex([:test, :test, :test, :test, :test])
