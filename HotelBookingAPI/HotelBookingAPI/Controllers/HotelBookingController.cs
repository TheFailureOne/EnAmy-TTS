using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using System.Speech;
using HotelBookingAPI.Models;
using HotelBookingAPI.Data;
using System.Speech.Synthesis;
using Microsoft.AspNetCore.StaticFiles;

namespace HotelBookingAPI.Controllers
{
    [Route("api/[controller]/[action]")]
    [ApiController]
    public class HotelBookingController : ControllerBase
    {
        private readonly ApiContext _context;
        SpeechSynthesizer synth = new SpeechSynthesizer();

        public HotelBookingController(ApiContext context)
        {
            _context = context;
        }

        [HttpPost]
        public async Task<IActionResult> CreateVoice(VoiceLine voiceLine)
        {
            string filepath = Path.Combine(Directory.GetCurrentDirectory(), "Voices", "result.wav");
            synth.SelectVoiceByHints(VoiceGender.Female);
            synth.SetOutputToWaveFile(filepath);
            synth.Speak(voiceLine.Line);
            synth.SetOutputToNull();
            var provider = new FileExtensionContentTypeProvider();
            if (!provider.TryGetContentType(filepath, out var contenttype))
            {
                contenttype = "application/octet-stream";
            }

            var bytes = await System.IO.File.ReadAllBytesAsync(filepath);
            return File(bytes, contenttype, Path.GetFileName(filepath));
        }

        //[HttpGet]
        //[Route("DownloadFile")]
        //public async Task<IActionResult> DownloadFile(string filename)
        //{
        //    var filepath = Path.Combine(Directory.GetCurrentDirectory(), "Voices", filename);
//
       //     var provider = new FileExtensionContentTypeProvider();
       //     if(!provider.TryGetContentType(filepath, out var contenttype))
       //     {
       //         contenttype = "application/octet-stream";
        //    }
//
        //    var bytes = await System.IO.File.ReadAllBytesAsync(filepath);
        //    return File(bytes, contenttype, Path.GetFileName(filepath));
        //}


        //Create/Edit
        [HttpPost]
        public JsonResult CreateEdit(HotelBooking booking)
        {
            if (booking.Id == 0)
            {
                _context.Bookings.Add(booking);
            }
            else
            {
                var bookingInDb = _context.Bookings.Find(booking.Id);

                if (bookingInDb == null)
                    return new JsonResult(NotFound());

                bookingInDb = booking;
            }

            _context.SaveChanges();

            return new JsonResult(Ok(booking));
        }

        // Get
        [HttpGet]
        public JsonResult Get(int id) {
            var result = _context.Bookings.Find(id);

            if (result == null)
                return new JsonResult(NotFound());

            return new JsonResult(Ok(result));
        }

        [HttpDelete]
        public JsonResult Delete(int id) {
            var result = _context.Bookings.Find(id);

            if (result == null)
                return new JsonResult(NotFound());

            _context.Bookings.Remove(result);
            _context.SaveChanges();

            return new JsonResult(NoContent());
        }

        //Get all
        [HttpGet()]
        public JsonResult GetAll()
        {
            var result = _context.Bookings.ToList();

            return new JsonResult(Ok(result));
        }
    }
}
