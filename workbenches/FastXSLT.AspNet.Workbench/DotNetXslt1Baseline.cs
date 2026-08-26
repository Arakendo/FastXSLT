using System.Globalization;
using System.Text;
using System.Xml;
using System.Xml.XPath;
using System.Xml.Xsl;

public sealed class DotNetXslt1Baseline
{
    private readonly XPathDocument _source;
    private readonly XslCompiledTransform _stylesheet;

    private DotNetXslt1Baseline(XPathDocument source, XslCompiledTransform stylesheet)
    {
        _source = source;
        _stylesheet = stylesheet;
    }

    public static DotNetXslt1Baseline Create(byte[] source, byte[] stylesheet)
    {
        using var sourceStream = new MemoryStream(source, writable: false);
        using var sourceReader = XmlReader.Create(sourceStream, ReaderSettings());
        var preparedSource = new XPathDocument(sourceReader);

        using var stylesheetStream = new MemoryStream(stylesheet, writable: false);
        using var stylesheetReader = XmlReader.Create(stylesheetStream, ReaderSettings());
        var compiledStylesheet = new XslCompiledTransform();
        compiledStylesheet.Load(stylesheetReader, XsltSettings.Default, null);
        return new DotNetXslt1Baseline(preparedSource, compiledStylesheet);
    }

    public static ExactStylesheetProbe ProbeExactStylesheet(byte[] source, byte[] stylesheet)
    {
        try
        {
            var transform = Create(source, stylesheet);
            return new ExactStylesheetProbe(true, transform.Transform());
        }
        catch (XsltException failure)
        {
            return new ExactStylesheetProbe(false, failure.Message);
        }
    }

    public string Transform()
    {
        using var output = new Utf8StringWriter(CultureInfo.InvariantCulture);
        using var writer = XmlWriter.Create(output, _stylesheet.OutputSettings);
        _stylesheet.Transform(_source, arguments: null, writer);
        writer.Flush();
        return output.ToString();
    }

    private static XmlReaderSettings ReaderSettings() => new()
    {
        DtdProcessing = DtdProcessing.Prohibit,
        XmlResolver = null
    };

    private sealed class Utf8StringWriter(IFormatProvider formatProvider)
        : StringWriter(formatProvider)
    {
        public override Encoding Encoding => Encoding.UTF8;
    }
}

public sealed record ExactStylesheetProbe(bool Executed, string Detail);
