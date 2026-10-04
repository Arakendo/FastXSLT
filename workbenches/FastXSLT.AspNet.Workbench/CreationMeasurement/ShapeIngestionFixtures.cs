using System.Globalization;
using System.Text;

// Synthetic anatomy shapes, not imported standards cases or a consumer distribution.
internal static class ShapeIngestionFixtures
{
    internal static byte[] Style => Encoding.UTF8.GetBytes(
        "<xsl:stylesheet xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\" version=\"1.0\">" +
        "<xsl:output omit-xml-declaration=\"yes\"/><xsl:template match=\"/\">" +
        "<xsl:copy-of select=\"/\"/></xsl:template></xsl:stylesheet>");

    internal static IReadOnlyList<PersistentIngestionProbe.Fixture> Create()
    {
        static string Repeat(string value, int count) => string.Concat(Enumerable.Repeat(value, count));
        static string Number(int value) => value.ToString(CultureInfo.InvariantCulture);
        var attributes = string.Concat(Enumerable.Range(0, 16).Select(i => $" a{Number(i)}=\"same\""));
        var namespaces = string.Concat(Enumerable.Range(0, 8).Select(i => $" xmlns:p{Number(i)}=\"urn:ns:{Number(i)}\""));
        var sources = new (string Name, string Xml)[]
        {
            ("wide", "<r>" + Repeat("<item code=\"same\">shared text</item>", 1000) + "</r>"),
            // Both current adapters enforce the existing default depth ceiling of 64.
            ("deep-64", Repeat("<n>", 64) + "leaf" + Repeat("</n>", 64)),
            ("attribute-heavy", "<r>" + Repeat($"<item{attributes}/>", 64) + "</r>"),
            ("text-heavy", "<r>" + Repeat("<item>" + new string('x', 4096) + "</item>", 64) + "</r>"),
            ("namespace-heavy", "<r>" + Repeat($"<p0:item{namespaces}/>", 128) + "</r>"),
            ("low-repetition", "<r>" + string.Concat(Enumerable.Range(0, 512).Select(i =>
                $"<item{Number(i)} code{Number(i)}=\"v{i.ToString("D4", CultureInfo.InvariantCulture)}\">" +
                $"t{i.ToString("D4", CultureInfo.InvariantCulture)}</item{Number(i)}>")) + "</r>"),
        };
        // Copied result elements serialize with explicit end tags on this path.
        // Keep original source bytes; expect the disclosed result serialization.
        return sources.Select(s => new PersistentIngestionProbe.Fixture(
            s.Name, Encoding.UTF8.GetBytes(s.Xml), s.Name switch
            {
                "attribute-heavy" => "<r>" + Repeat($"<item{attributes}></item>", 64) + "</r>",
                "namespace-heavy" => "<r>" + Repeat($"<p0:item{namespaces}></p0:item>", 128) + "</r>",
                _ => s.Xml,
            })).ToArray();
    }
}
