using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;

namespace Phenotype.Traceability
{
    /// <summary>
    /// Attribute to mark a test as tracing to a Feature Requirement (FR).
    /// </summary>
    [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
    public class TraceToAttribute : Attribute
    {
        public string[] FrIds { get; }

        public TraceToAttribute(params string[] frIds)
        {
            foreach (var frId in frIds)
            {
                if (!ValidateFrId(frId))
                {
                    throw new ArgumentException($"Invalid FR ID format: {frId}. Expected: FR-XXXX-NNN");
                }
            }
            FrIds = frIds;
        }

        private static bool ValidateFrId(string frId)
        {
            var pattern = new Regex(@"^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$");
            return pattern.IsMatch(frId);
        }
    }

    /// <summary>
    /// Helper class for FR traceability in tests.
    /// </summary>
    public static class Traceability
    {
        private static readonly Dictionary<string, List<string>> _traces = new();

        public static void TraceTo(string testName, params string[] frIds)
        {
            _traces[testName] = frIds.ToList();
        }

        public static IReadOnlyDictionary<string, List<string>> GetTraces()
        {
            return _traces;
        }

        public static bool ValidateFrId(string frId)
        {
            var pattern = new Regex(@"^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$");
            return pattern.IsMatch(frId);
        }
    }
}
