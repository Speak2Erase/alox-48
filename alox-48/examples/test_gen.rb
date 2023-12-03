def dump_to_hex(obj)
  e = Marshal.dump(obj)
  e = e.bytes.map { |b| "0x" + b.to_s(16).rjust(2, "0") }.join(", ")

  puts "&[#{e}];"
end

class BadData
  def initialize
    @good = ["hfjvhjvjhvhjvl"]
    @bad = { "oops": "something went wrong" }
    @after_bad = "im a string! nothing can go wrong here :)"
  end
end

dump_to_hex([BadData.new])
